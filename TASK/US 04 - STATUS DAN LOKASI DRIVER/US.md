# US 04 — STATUS DAN LOKASI DRIVER

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai driver yang disetujui, saya ingin mengatur status online dan mengirim lokasi terkini, agar dapat dipertimbangkan untuk pesanan di sekitar.

## API scope

- `PUT /api/v1/driver/availability`
- `PUT /api/v1/driver/location`
- `GET /api/v1/driver/trips/offers`

## Dependencies / keputusan

US 03; Redis memegang lokasi sementara dengan TTL; freshness dan frekuensi perlu dikonfigurasi.

## Acceptance Criteria cards

- [AC 04.01 — Atur status availability](AC/AC%2004.01%20-%20Atur%20status%20availability.md)
- [AC 04.02 — Kirim lokasi valid](AC/AC%2004.02%20-%20Kirim%20lokasi%20valid.md)
- [AC 04.03 — Singkirkan lokasi stale](AC/AC%2004.03%20-%20Singkirkan%20lokasi%20stale.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
