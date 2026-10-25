# US 01 — API LOGIN

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai pengguna, saya ingin login dengan nomor telepon terverifikasi, agar mengakses API sesuai perannya.

## API scope

- `POST /api/v1/auth/otp/request`
- `POST /api/v1/auth/otp/verify`
- `POST /api/v1/auth/token/refresh`
- `POST /api/v1/auth/logout`

## Dependencies / keputusan

Provider OTP belum dipilih; tentukan sebelum integrasi.

## Acceptance Criteria cards

- [AC 01.01 — Minta OTP](AC/AC%2001.01%20-%20Minta%20OTP.md)
- [AC 01.02 — Verifikasi OTP](AC/AC%2001.02%20-%20Verifikasi%20OTP.md)
- [AC 01.03 — Refresh dan logout](AC/AC%2001.03%20-%20Refresh%20dan%20logout.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
