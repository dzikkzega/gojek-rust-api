# US 02 — PROFIL PENUMPANG

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai penumpang, saya ingin melihat dan memperbarui profil sendiri, agar data akun tetap benar dan terlindungi.

## API scope

- `GET /api/v1/me`
- `PATCH /api/v1/me`

## Dependencies / keputusan

US 01 — API Login.

## Acceptance Criteria cards

- [AC 02.01 — Lihat profil](AC/AC%2002.01%20-%20Lihat%20profil.md)
- [AC 02.02 — Ubah profil](AC/AC%2002.02%20-%20Ubah%20profil.md)
- [AC 02.03 — Tolak akses tanpa sesi](AC/AC%2002.03%20-%20Tolak%20akses%20tanpa%20sesi.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
