# US 01 — API LOGIN

**Project:** GOJEK-rust-api  
**Status:** In Progress  
**Priority:** P0 — MVP

## User Story

Sebagai pengguna, saya ingin login dengan nomor telepon terverifikasi, agar mengakses API sesuai perannya.

## API scope

- `POST /api/v1/auth/otp/request` (handler: `src/auth.rs`)
- `POST /api/v1/auth/otp/verify` (handler: `src/auth.rs`)
- `POST /api/v1/auth/token/refresh` (handler: `src/auth.rs`)
- `POST /api/v1/auth/logout` (handler: `src/auth.rs`)

## Dependencies / keputusan

- Migrasi `auth_otp_challenges` telah diterapkan ke PostgreSQL lokal.
- Provider OTP SMS/WhatsApp produksi belum dipilih (D-05 Open). `DEV_STATIC_OTP` hanya tersedia untuk pengujian lokal.
- Access token memakai JWT HMAC-SHA256 (15 menit); refresh token acak disimpan sebagai hash SHA-256 dalam `auth_sessions`.

## Acceptance Criteria cards

- [AC 01.01 — Minta OTP](AC/AC%2001.01%20-%20Minta%20OTP.md)
- [AC 01.02 — Verifikasi OTP](AC/AC%2001.02%20-%20Verifikasi%20OTP.md)
- [AC 01.03 — Refresh dan logout](AC/AC%2001.03%20-%20Refresh%20dan%20logout.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
