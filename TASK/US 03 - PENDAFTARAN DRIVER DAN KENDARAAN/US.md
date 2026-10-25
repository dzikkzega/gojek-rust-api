# US 03 — PENDAFTARAN DRIVER DAN KENDARAAN

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai calon driver, saya ingin mengajukan pendaftaran dan mendaftarkan kendaraan, agar dapat menjadi driver setelah persetujuan admin.

## API scope

- `POST /api/v1/driver/applications`
- `GET /api/v1/driver/application`
- `POST /api/v1/driver/application/documents`
- `POST /api/v1/driver/application/submit`
- `GET/POST /api/v1/driver/vehicles`

## Dependencies / keputusan

US 01; kebijakan verifikasi, dokumen, storage, dan retensi perlu diputuskan; persetujuan admin di US 10.

## Acceptance Criteria cards

- [AC 03.01 — Ajukan pendaftaran](AC/AC%2003.01%20-%20Ajukan%20pendaftaran.md)
- [AC 03.02 — Kirim dokumen dan submit](AC/AC%2003.02%20-%20Kirim%20dokumen%20dan%20submit.md)
- [AC 03.03 — Kelola kendaraan sendiri](AC/AC%2003.03%20-%20Kelola%20kendaraan%20sendiri.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
