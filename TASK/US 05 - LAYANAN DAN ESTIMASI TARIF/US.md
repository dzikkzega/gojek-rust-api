# US 05 — LAYANAN DAN ESTIMASI TARIF

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai penumpang, saya ingin memilih layanan dan meminta estimasi tarif perjalanan, agar mengetahui perkiraan sebelum memesan.

## API scope

- `GET /api/v1/ride-types`
- `POST /api/v1/ride-quotes`
- `GET /api/v1/ride-quotes/{quote_id}`

## Dependencies / keputusan

Aturan tarif, mata uang, area, provider rute, dan masa berlaku quote perlu ditetapkan sebelum produksi.

## Acceptance Criteria cards

- [AC 05.01 — Lihat layanan aktif](AC/AC%2005.01%20-%20Lihat%20layanan%20aktif.md)
- [AC 05.02 — Validasi titik dan area](AC/AC%2005.02%20-%20Validasi%20titik%20dan%20area.md)
- [AC 05.03 — Buat quote berjangka](AC/AC%2005.03%20-%20Buat%20quote%20berjangka.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
