# US 07 — STATUS DAN TRACKING PERJALANAN

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai penumpang atau driver peserta, saya ingin melihat status dan lokasi terkini perjalanan, agar mengetahui progres perjalanan secara aman.

## API scope

- `POST /api/v1/trips/{trip_id}/cancel`
- `POST /api/v1/driver/trips/{trip_id}/arrive`
- `POST /api/v1/driver/trips/{trip_id}/start`
- `POST /api/v1/driver/trips/{trip_id}/complete`
- `GET /api/v1/trips/{trip_id}/tracking`

## Dependencies / keputusan

US 06; kebijakan cancel, kode mulai, batas percobaan, dan freshness perlu dikonfigurasi.

## Acceptance Criteria cards

- [AC 07.01 — Terapkan transisi status](AC/AC%2007.01%20-%20Terapkan%20transisi%20status.md)
- [AC 07.02 — Batalkan sebagai peserta](AC/AC%2007.02%20-%20Batalkan%20sebagai%20peserta.md)
- [AC 07.03 — Verifikasi mulai dan tracking](AC/AC%2007.03%20-%20Verifikasi%20mulai%20dan%20tracking.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
