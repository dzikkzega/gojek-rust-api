# US 08 — RIWAYAT DAN PEMBAYARAN TUNAI

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai penumpang atau driver peserta, saya ingin melihat riwayat dan status pembayaran tunai, agar memiliki catatan perjalanan dan jumlah yang tercatat.

## API scope

- `GET /api/v1/trips`
- `GET /api/v1/trips/{trip_id}`
- `GET /api/v1/trips/{trip_id}/payment`
- `GET /api/v1/payment-methods`

## Dependencies / keputusan

US 07; MVP mencatat pembayaran cash, bukan memproses transfer uang.

## Acceptance Criteria cards

- [AC 08.01 — Baca riwayat sendiri](AC/AC%2008.01%20-%20Baca%20riwayat%20sendiri.md)
- [AC 08.02 — Catat pembayaran cash](AC/AC%2008.02%20-%20Catat%20pembayaran%20cash.md)
- [AC 08.03 — Baca rincian pembayaran](AC/AC%2008.03%20-%20Baca%20rincian%20pembayaran.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
