# US 10 — ADMIN DAN OPERASIONAL

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai admin atau operator berizin, saya ingin mengelola pendaftaran, tarif, area, dan pemeriksaan operasional, agar layanan dapat dijalankan dengan akses terkendali.

## API scope

- `/api/v1/admin/driver-applications`
- `/api/v1/admin/trips`
- `/api/v1/admin/service-areas`
- `/api/v1/admin/fare-rules`
- `GET /api/v1/health`
- `GET /api/v1/ready`

## Dependencies / keputusan

Akun admin/permission disediakan terkontrol; tidak ada pendaftaran admin publik.

## Acceptance Criteria cards

- [AC 10.01 — Review driver dengan audit](AC/AC%2010.01%20-%20Review%20driver%20dengan%20audit.md)
- [AC 10.02 — Kelola area dan tarif](AC/AC%2010.02%20-%20Kelola%20area%20dan%20tarif.md)
- [AC 10.03 — Periksa trip dengan izin](AC/AC%2010.03%20-%20Periksa%20trip%20dengan%20izin.md)
- [AC 10.04 — Health, readiness, dan log aman](AC/AC%2010.04%20-%20Health%2C%20readiness%2C%20dan%20log%20aman.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
