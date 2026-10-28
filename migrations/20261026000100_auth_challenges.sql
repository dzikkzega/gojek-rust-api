CREATE TYPE otp_challenge_status AS ENUM ('active', 'consumed', 'locked', 'expired');

CREATE TABLE auth_otp_challenges (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_e164 text NOT NULL,
    code_hash bytea NOT NULL,
    status otp_challenge_status NOT NULL DEFAULT 'active',
    attempt_count smallint NOT NULL DEFAULT 0,
    max_attempts smallint NOT NULL,
    expires_at timestamptz NOT NULL,
    resend_after timestamptz NOT NULL,
    request_ip_hash bytea,
    created_at timestamptz NOT NULL DEFAULT now(),
    consumed_at timestamptz,
    CONSTRAINT auth_otp_phone_nonempty CHECK (length(trim(phone_e164)) > 0),
    CONSTRAINT auth_otp_attempt_count CHECK (attempt_count >= 0 AND max_attempts > 0 AND attempt_count <= max_attempts),
    CONSTRAINT auth_otp_expiry CHECK (expires_at > created_at),
    CONSTRAINT auth_otp_resend CHECK (resend_after >= created_at),
    CONSTRAINT auth_otp_consumed CHECK ((status = 'consumed') = (consumed_at IS NOT NULL))
);

CREATE INDEX auth_otp_phone_active_idx ON auth_otp_challenges(phone_e164, created_at DESC)
    WHERE status = 'active';
CREATE INDEX auth_otp_expiry_idx ON auth_otp_challenges(expires_at)
    WHERE status = 'active';
