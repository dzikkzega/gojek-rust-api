-- Switch authentication identity from phone OTP to email OTP.
-- Existing verified email values are normalized. Users without email remain
-- unverified and cannot sign in until they complete email verification.

ALTER TABLE users
    ALTER COLUMN phone_e164 DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS email_normalized text,
    ADD COLUMN IF NOT EXISTS email_verified_at timestamptz;

UPDATE users
SET email_normalized = lower(trim(email))
WHERE email_normalized IS NULL AND email IS NOT NULL AND length(trim(email)) > 0;

DROP INDEX IF EXISTS users_email_normalized_uq;
CREATE UNIQUE INDEX users_email_normalized_uq
    ON users(email_normalized);

ALTER TABLE auth_otp_challenges
    ADD COLUMN email_normalized text;

-- Existing challenges were phone OTPs; remove unassociated phone challenges before
-- requiring email_normalized on all challenges.
DELETE FROM auth_otp_challenges
WHERE email_normalized IS NULL;

ALTER TABLE auth_otp_challenges
    ALTER COLUMN phone_e164 DROP NOT NULL,
    ALTER COLUMN email_normalized SET NOT NULL,
    ADD CONSTRAINT auth_otp_identifier_exactly_one
        CHECK ((phone_e164 IS NULL) <> (email_normalized IS NULL)),
    ADD CONSTRAINT auth_otp_email_normalized_format
        CHECK (email_normalized ~ '^[^[:space:]@]+@[^[:space:]@]+\.[^[:space:]@]+$');

DROP INDEX IF EXISTS auth_otp_phone_active_idx;
CREATE INDEX auth_otp_email_active_idx
    ON auth_otp_challenges(email_normalized, created_at DESC)
    WHERE status = 'active';
CREATE UNIQUE INDEX auth_otp_one_active_per_email_idx
    ON auth_otp_challenges(email_normalized)
    WHERE status = 'active';
