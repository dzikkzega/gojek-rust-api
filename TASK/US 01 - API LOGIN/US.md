# US 01 — API LOGIN

**Project:** GOJEK-rust-api  
**Status:** Done (local Mailpit; production relay D-05 Open)  
**Priority:** P0 — MVP

## User Story

Sebagai pengguna, saya ingin login dengan email terverifikasi, agar mengakses API sesuai perannya.

## API scope

- `POST /api/v1/auth/otp/request` (handler: `src/auth.rs`)
- `POST /api/v1/auth/otp/verify` (handler: `src/auth.rs`)
- `POST /api/v1/auth/token/refresh` (handler: `src/auth.rs`)
- `POST /api/v1/auth/logout` (handler: `src/auth.rs`)

## Dependencies / keputusan

- Migrasi forward `20261028000100_email_otp.sql` memindahkan identitas login ke `email_normalized` dan mempertahankan telepon sebagai kontak opsional.
- SMTP lokal menggunakan Mailpit (`127.0.0.1:1025`, inbox UI `127.0.0.1:8025`); relay produksi dan kebijakan verifikasi email masih D-05 Open.
- Access token memakai JWT HMAC-SHA256 (15 menit); refresh token acak disimpan sebagai hash SHA-256 dalam `auth_sessions`; replay refresh mencabut seluruh sesi user; maksimal 8 sesi aktif per akun.

## Acceptance Criteria cards

- [AC 01.01 — Minta OTP](AC/AC%2001.01%20-%20Minta%20OTP.md)
- [AC 01.02 — Verifikasi OTP](AC/AC%2001.02%20-%20Verifikasi%20OTP.md)
- [AC 01.03 — Refresh dan logout](AC/AC%2001.03%20-%20Refresh%20dan%20logout.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
