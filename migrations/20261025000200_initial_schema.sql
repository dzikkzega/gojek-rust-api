-- Core ride-hailing MVP schema. PostGIS is enabled by the preceding migration.

CREATE TYPE account_status AS ENUM ('active', 'suspended', 'deleted');
CREATE TYPE user_role AS ENUM ('rider', 'driver', 'admin', 'support');
CREATE TYPE rating_direction AS ENUM ('rider_to_driver', 'driver_to_rider');
CREATE TYPE driver_application_status AS ENUM ('draft', 'pending', 'approved', 'rejected');
CREATE TYPE driver_status AS ENUM ('offline', 'online', 'suspended');
CREATE TYPE vehicle_type AS ENUM ('motorcycle', 'car');
CREATE TYPE quote_status AS ENUM ('active', 'consumed', 'expired');
CREATE TYPE trip_status AS ENUM (
    'searching', 'driver_assigned', 'driver_arrived', 'in_progress',
    'completed', 'cancelled', 'no_driver_found'
);
CREATE TYPE trip_actor AS ENUM ('system', 'rider', 'driver', 'admin');
CREATE TYPE payment_method AS ENUM ('cash');
CREATE TYPE payment_status AS ENUM ('pending', 'paid', 'failed', 'refunded');
CREATE TYPE document_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_e164 text NOT NULL UNIQUE,
    display_name text NOT NULL DEFAULT '',
    email text UNIQUE,
    status account_status NOT NULL DEFAULT 'active',
    role user_role NOT NULL DEFAULT 'rider',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz,
    CONSTRAINT users_phone_nonempty CHECK (length(trim(phone_e164)) > 0),
    CONSTRAINT users_display_name_length CHECK (length(display_name) <= 120)
);

CREATE TABLE auth_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash text NOT NULL UNIQUE,
    device_label text,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT auth_sessions_expiry_after_creation CHECK (expires_at > created_at)
);
CREATE INDEX auth_sessions_user_id_idx ON auth_sessions(user_id);
CREATE INDEX auth_sessions_active_expiry_idx ON auth_sessions(expires_at) WHERE revoked_at IS NULL;

CREATE TABLE driver_applications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL UNIQUE REFERENCES users(id) ON DELETE RESTRICT,
    status driver_application_status NOT NULL DEFAULT 'draft',
    submitted_at timestamptz,
    reviewed_at timestamptz,
    reviewed_by uuid REFERENCES users(id) ON DELETE SET NULL,
    rejection_reason text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (id, user_id),
    CONSTRAINT driver_application_review_consistency CHECK (
        (status IN ('approved', 'rejected') AND reviewed_at IS NOT NULL)
        OR (status NOT IN ('approved', 'rejected') AND reviewed_at IS NULL)
    ),
    CONSTRAINT driver_application_rejection_reason CHECK (
        status <> 'rejected' OR length(trim(rejection_reason)) > 0
    )
);
CREATE INDEX driver_applications_status_created_idx ON driver_applications(status, created_at);

CREATE TABLE driver_profiles (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE RESTRICT,
    application_id uuid NOT NULL UNIQUE,
    status driver_status NOT NULL DEFAULT 'offline',
    approved_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT driver_profiles_application_owner_fk
        FOREIGN KEY (application_id, user_id)
        REFERENCES driver_applications(id, user_id) ON DELETE RESTRICT
);
CREATE INDEX driver_profiles_status_idx ON driver_profiles(status);

CREATE TABLE driver_documents (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES driver_applications(id) ON DELETE RESTRICT,
    document_type text NOT NULL,
    storage_key text NOT NULL UNIQUE,
    status document_status NOT NULL DEFAULT 'pending',
    reviewed_by uuid REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at timestamptz,
    rejection_reason text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT driver_documents_type_nonempty CHECK (length(trim(document_type)) > 0),
    CONSTRAINT driver_documents_review_consistency CHECK (
        (status = 'pending' AND reviewed_at IS NULL)
        OR (status <> 'pending' AND reviewed_at IS NOT NULL)
    )
);
CREATE INDEX driver_documents_application_idx ON driver_documents(application_id);

CREATE TABLE vehicles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    driver_id uuid NOT NULL REFERENCES driver_profiles(user_id) ON DELETE RESTRICT,
    vehicle_type vehicle_type NOT NULL,
    registration_number text NOT NULL UNIQUE,
    make text NOT NULL,
    model text NOT NULL,
    color text NOT NULL,
    year smallint,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (id, driver_id),
    UNIQUE (id, vehicle_type),
    CONSTRAINT vehicles_registration_nonempty CHECK (length(trim(registration_number)) > 0),
    CONSTRAINT vehicles_year_range CHECK (year IS NULL OR year BETWEEN 1950 AND 2200)
);
CREATE INDEX vehicles_driver_active_idx ON vehicles(driver_id) WHERE is_active;

CREATE TABLE service_areas (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    boundary geography(MultiPolygon, 4326) NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT service_areas_name_nonempty CHECK (length(trim(name)) > 0)
);
CREATE INDEX service_areas_boundary_gix ON service_areas USING gist(boundary);

CREATE TABLE fare_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    service_area_id uuid NOT NULL REFERENCES service_areas(id) ON DELETE RESTRICT,
    vehicle_type vehicle_type NOT NULL,
    currency char(3) NOT NULL,
    base_fare_minor bigint NOT NULL,
    per_km_minor bigint NOT NULL,
    per_minute_minor bigint NOT NULL DEFAULT 0,
    minimum_fare_minor bigint NOT NULL,
    effective_from timestamptz NOT NULL,
    effective_until timestamptz,
    created_by uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT fare_rules_currency_upper CHECK (currency ~ '^[A-Z]{3}$'),
    CONSTRAINT fare_rules_amounts_nonnegative CHECK (
        base_fare_minor >= 0 AND per_km_minor >= 0 AND per_minute_minor >= 0 AND minimum_fare_minor >= 0
    ),
    CONSTRAINT fare_rules_effective_range CHECK (effective_until IS NULL OR effective_until > effective_from),
    UNIQUE (id, service_area_id, vehicle_type),
    UNIQUE (id, currency),
    UNIQUE (id, service_area_id, vehicle_type, currency)
);
CREATE INDEX fare_rules_lookup_idx ON fare_rules(service_area_id, vehicle_type, effective_from DESC);

CREATE TABLE ride_quotes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    rider_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    service_area_id uuid NOT NULL REFERENCES service_areas(id) ON DELETE RESTRICT,
    fare_rule_id uuid NOT NULL,
    vehicle_type vehicle_type NOT NULL,
    pickup geography(Point, 4326) NOT NULL,
    destination geography(Point, 4326) NOT NULL,
    estimated_distance_m integer NOT NULL,
    estimated_duration_s integer NOT NULL,
    estimated_fare_minor bigint NOT NULL,
    currency char(3) NOT NULL,
    status quote_status NOT NULL DEFAULT 'active',
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ride_quotes_fare_rule_fk FOREIGN KEY (fare_rule_id, service_area_id, vehicle_type, currency)
        REFERENCES fare_rules(id, service_area_id, vehicle_type, currency) ON DELETE RESTRICT,
    CONSTRAINT ride_quotes_distance_positive CHECK (estimated_distance_m > 0),
    CONSTRAINT ride_quotes_duration_positive CHECK (estimated_duration_s > 0),
    CONSTRAINT ride_quotes_fare_nonnegative CHECK (estimated_fare_minor >= 0),
    CONSTRAINT ride_quotes_expiry_after_creation CHECK (expires_at > created_at),
    CONSTRAINT ride_quotes_currency_upper CHECK (currency ~ '^[A-Z]{3}$'),
    UNIQUE (id, rider_id, service_area_id, fare_rule_id, vehicle_type, estimated_fare_minor, currency)
);
CREATE INDEX ride_quotes_rider_created_idx ON ride_quotes(rider_id, created_at DESC);
CREATE INDEX ride_quotes_expiry_idx ON ride_quotes(expires_at) WHERE status = 'active';

CREATE TABLE trips (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    rider_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    driver_id uuid REFERENCES driver_profiles(user_id) ON DELETE RESTRICT,
    vehicle_id uuid,
    quote_id uuid NOT NULL UNIQUE,
    service_area_id uuid NOT NULL,
    fare_rule_id uuid NOT NULL,
    vehicle_type vehicle_type NOT NULL,
    pickup geography(Point, 4326) NOT NULL,
    destination geography(Point, 4326) NOT NULL,
    status trip_status NOT NULL DEFAULT 'searching',
    estimated_fare_minor bigint NOT NULL,
    final_fare_minor bigint,
    currency char(3) NOT NULL,
    cancellation_reason text,
    idempotency_key text NOT NULL,
    requested_at timestamptz NOT NULL DEFAULT now(),
    assigned_at timestamptz,
    arrived_at timestamptz,
    started_at timestamptz,
    completed_at timestamptz,
    cancelled_at timestamptz,
    CONSTRAINT trips_rider_idempotency_key UNIQUE (rider_id, idempotency_key),
    UNIQUE (id, rider_id, driver_id),
    CONSTRAINT trips_fare_rule_fk FOREIGN KEY (fare_rule_id, service_area_id, vehicle_type, currency)
        REFERENCES fare_rules(id, service_area_id, vehicle_type, currency) ON DELETE RESTRICT,
    CONSTRAINT trips_quote_snapshot_fk FOREIGN KEY (quote_id, rider_id, service_area_id, fare_rule_id, vehicle_type, estimated_fare_minor, currency)
        REFERENCES ride_quotes(id, rider_id, service_area_id, fare_rule_id, vehicle_type, estimated_fare_minor, currency) ON DELETE RESTRICT,
    CONSTRAINT trips_vehicle_driver_fk FOREIGN KEY (vehicle_id, driver_id)
        REFERENCES vehicles(id, driver_id) ON DELETE RESTRICT,
    CONSTRAINT trips_vehicle_type_fk FOREIGN KEY (vehicle_id, vehicle_type)
        REFERENCES vehicles(id, vehicle_type) ON DELETE RESTRICT,
    CONSTRAINT trips_assigned_pair_check CHECK (
        (driver_id IS NULL AND vehicle_id IS NULL)
        OR (driver_id IS NOT NULL AND vehicle_id IS NOT NULL)
    ),
    CONSTRAINT trips_fares_nonnegative CHECK (
        estimated_fare_minor >= 0 AND (final_fare_minor IS NULL OR final_fare_minor >= 0)
    ),
    CONSTRAINT trips_currency_upper CHECK (currency ~ '^[A-Z]{3}$'),
    CONSTRAINT trips_idempotency_key_nonempty CHECK (length(trim(idempotency_key)) > 0),
    CONSTRAINT trips_state_consistency CHECK (
        (status <> 'completed' OR (completed_at IS NOT NULL AND final_fare_minor IS NOT NULL))
        AND (status <> 'cancelled' OR (cancelled_at IS NOT NULL AND length(trim(cancellation_reason)) > 0))
        AND (status <> 'driver_assigned' OR (driver_id IS NOT NULL AND assigned_at IS NOT NULL))
        AND (status <> 'driver_arrived' OR (driver_id IS NOT NULL AND arrived_at IS NOT NULL))
        AND (status <> 'in_progress' OR (driver_id IS NOT NULL AND started_at IS NOT NULL))
        AND (status <> 'no_driver_found' OR driver_id IS NULL)
        AND (driver_id IS NULL OR assigned_at IS NOT NULL)
        AND (arrived_at IS NULL OR assigned_at IS NOT NULL)
        AND (started_at IS NULL OR arrived_at IS NOT NULL)
        AND (completed_at IS NULL OR started_at IS NOT NULL)
        AND (cancelled_at IS NULL OR status = 'cancelled')
    )
);
CREATE INDEX trips_rider_requested_idx ON trips(rider_id, requested_at DESC);
CREATE INDEX trips_driver_requested_idx ON trips(driver_id, requested_at DESC) WHERE driver_id IS NOT NULL;
CREATE INDEX trips_status_requested_idx ON trips(status, requested_at);
CREATE INDEX trips_pickup_gix ON trips USING gist(pickup);
CREATE INDEX trips_destination_gix ON trips USING gist(destination);

CREATE TABLE trip_events (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    trip_id uuid NOT NULL REFERENCES trips(id) ON DELETE RESTRICT,
    actor_type trip_actor NOT NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    from_status trip_status,
    to_status trip_status NOT NULL,
    details jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT trip_events_status_change CHECK (from_status IS NULL OR from_status <> to_status)
);
CREATE INDEX trip_events_trip_created_idx ON trip_events(trip_id, created_at);

CREATE TABLE payments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id uuid NOT NULL UNIQUE REFERENCES trips(id) ON DELETE RESTRICT,
    method payment_method NOT NULL DEFAULT 'cash',
    status payment_status NOT NULL DEFAULT 'pending',
    amount_minor bigint NOT NULL,
    currency char(3) NOT NULL,
    paid_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT payments_amount_nonnegative CHECK (amount_minor >= 0),
    CONSTRAINT payments_currency_upper CHECK (currency ~ '^[A-Z]{3}$'),
    CONSTRAINT payments_paid_timestamp CHECK (
        (status IN ('paid', 'refunded') AND paid_at IS NOT NULL)
        OR (status NOT IN ('paid', 'refunded') AND paid_at IS NULL)
    )
);

CREATE TABLE ratings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id uuid NOT NULL,
    rider_id uuid NOT NULL,
    driver_id uuid NOT NULL,
    direction rating_direction NOT NULL,
    score smallint NOT NULL,
    comment text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ratings_one_per_trip_direction UNIQUE (trip_id, direction),
    CONSTRAINT ratings_trip_participants_fk FOREIGN KEY (trip_id, rider_id, driver_id)
        REFERENCES trips(id, rider_id, driver_id) ON DELETE RESTRICT,
    CONSTRAINT ratings_score_range CHECK (score BETWEEN 1 AND 5),
    CONSTRAINT ratings_comment_length CHECK (comment IS NULL OR length(comment) <= 1000)
);
CREATE INDEX ratings_driver_created_idx ON ratings(driver_id, created_at DESC);
CREATE INDEX ratings_rider_created_idx ON ratings(rider_id, created_at DESC);

CREATE TABLE audit_logs (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    action text NOT NULL,
    target_type text NOT NULL,
    target_id text NOT NULL,
    reason text,
    request_id text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT audit_action_nonempty CHECK (length(trim(action)) > 0),
    CONSTRAINT audit_target_type_nonempty CHECK (length(trim(target_type)) > 0),
    CONSTRAINT audit_target_id_nonempty CHECK (length(trim(target_id)) > 0)
);
CREATE INDEX audit_logs_target_created_idx ON audit_logs(target_type, target_id, created_at DESC);
CREATE INDEX audit_logs_actor_created_idx ON audit_logs(actor_user_id, created_at DESC);
