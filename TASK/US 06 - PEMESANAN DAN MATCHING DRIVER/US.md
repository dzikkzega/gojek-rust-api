# US 06 — PEMESANAN DAN MATCHING DRIVER

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai penumpang, saya ingin memesan perjalanan dan mendapatkan satu driver, agar pesanan diproses tanpa duplikasi atau double assignment.

## API scope

- `POST /api/v1/trips`
- `POST /api/v1/driver/trips/{trip_id}/accept`
- `POST /api/v1/driver/trips/{trip_id}/decline`
- `GET /api/v1/driver/trips/offers`

## Dependencies / keputusan

US 04 dan US 05; radius, urutan kandidat, serta timeout matching harus dikonfigurasi.

## Acceptance Criteria cards

- [AC 06.01 — Buat booking idempotent](AC/AC%2006.01%20-%20Buat%20booking%20idempotent.md)
- [AC 06.02 — Pilih kandidat sesuai syarat](AC/AC%2006.02%20-%20Pilih%20kandidat%20sesuai%20syarat.md)
- [AC 06.03 — Klaim atomik dan timeout](AC/AC%2006.03%20-%20Klaim%20atomik%20dan%20timeout.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
