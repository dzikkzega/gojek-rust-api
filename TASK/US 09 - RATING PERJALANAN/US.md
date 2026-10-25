# US 09 — RATING PERJALANAN

**Project:** GOJEK-rust-api  
**Status:** Todo  
**Priority:** P0 — MVP

## User Story

Sebagai peserta perjalanan, saya ingin memberi penilaian setelah perjalanan selesai, agar kualitas layanan dapat dievaluasi.

## API scope

- `POST /api/v1/trips/{trip_id}/rating`

## Dependencies / keputusan

US 07; skala MVP 1–5; komentar opsional dan dibatasi panjangnya.

## Acceptance Criteria cards

- [AC 09.01 — Beri rating trip selesai](AC/AC%2009.01%20-%20Beri%20rating%20trip%20selesai.md)
- [AC 09.02 — Cegah rating ganda](AC/AC%2009.02%20-%20Cegah%20rating%20ganda.md)
- [AC 09.03 — Tolak akses rating silang](AC/AC%2009.03%20-%20Tolak%20akses%20rating%20silang.md)

## Definition of Done

- Semua AC di folder `AC/` selesai dan diuji.
- Otorisasi, validasi, dan error mengikuti [PRD](../../PRD.md).
- Data transaksi permanen di PostgreSQL; Redis hanya untuk lokasi/cache sementara.
- Tidak ada OTP, token, kredensial, atau koordinat presisi di log.
