# Relasi database — MVP

ERD berikut merangkum relasi pada migrations hingga [`20261028000100_email_otp.sql`](../migrations/20261028000100_email_otp.sql). `||` = satu, `o|` = opsional satu, `o{` = banyak.

```mermaid
erDiagram
    USERS ||--o{ AUTH_SESSIONS : owns
    USERS ||--o| DRIVER_APPLICATIONS : applies
    USERS ||--o| DRIVER_PROFILES : identity
    USERS ||--o{ RIDE_QUOTES : requests
    USERS ||--o{ TRIPS : rides
    USERS o|--o{ DRIVER_APPLICATIONS : reviews
    USERS o|--o{ DRIVER_DOCUMENTS : reviews
    USERS o|--o{ FARE_RULES : creates
    USERS o|--o{ TRIP_EVENTS : acts
    USERS o|--o{ AUDIT_LOGS : acts

    DRIVER_APPLICATIONS ||--o{ DRIVER_DOCUMENTS : includes
    DRIVER_APPLICATIONS ||--o| DRIVER_PROFILES : approved_as
    USERS ||--o{ RATINGS : rider_identity
    USERS ||--o{ RATINGS : driver_identity
    DRIVER_PROFILES ||--o{ VEHICLES : owns

    SERVICE_AREAS ||--o{ FARE_RULES : defines
    SERVICE_AREAS ||--o{ RIDE_QUOTES : covers
    SERVICE_AREAS ||--o{ TRIPS : covers
    FARE_RULES ||--o{ RIDE_QUOTES : prices
    FARE_RULES ||--o{ TRIPS : snapshots

    RIDE_QUOTES ||--o| TRIPS : consumed_once
    DRIVER_PROFILES o|--o{ TRIPS : assigned_driver
    VEHICLES o|--o{ TRIPS : assigned_vehicle
    TRIPS ||--o{ TRIP_EVENTS : records
    TRIPS ||--o| PAYMENTS : has_payment
    TRIPS ||--o{ RATINGS : receives
```

## Hubungan penting

- Login memakai `users.email_normalized` yang unik dan `email_verified_at`; `phone_e164` kini opsional dan tidak dipakai untuk autentikasi.
- Sebelum akun dibuat, OTP email disimpan sebagai challenge di `auth_otp_challenges`, berisi `email_normalized`, `code_hash`, status, attempt count, expiry, dan resend cooldown. Challenge menyimpan email, bukan FK ke `users`, agar email baru dapat diverifikasi sebelum membuat akun.
- `users` adalah identitas akun; role disimpan sebagai enum `users.role` (satu role per akun pada MVP). Pengguna dapat menjadi penumpang dan, setelah persetujuan, memiliki `driver_profile`.
- Setiap pengguna memiliki paling banyak satu `driver_application`. Aplikasi menyimpan banyak metadata dokumen dan dapat menghasilkan satu profil driver untuk pengguna yang sama.
- Setiap driver profile dapat memiliki banyak kendaraan. Composite FK trip memastikan kendaraan yang ditugaskan benar-benar milik driver itu dan sesuai tipe kendaraan.
- Area layanan memiliki aturan tarif berversi. Quote dan trip mengikat area, tipe kendaraan, mata uang, dan fare rule yang sama.
- Satu quote hanya dapat dikonsumsi satu trip. Composite FK juga memastikan rider dan snapshot tarif trip cocok dengan quote.
- `UNIQUE (rider_id, idempotency_key)` mencegah booking ganda saat retry. Klaim driver bersamaan tetap memerlukan transaksi/conditional update di aplikasi.
- Trip memiliki banyak status event, maksimal satu payment, dan maksimal satu rating per arah (`rider_to_driver` / `driver_to_rider`). Rating menyimpan pasangan peserta dari trip.
- PostGIS menyimpan batas area serta titik pickup/destination historis. Lokasi driver live berada di Redis dengan TTL, bukan di tabel PostgreSQL.
- `ON DELETE RESTRICT` melindungi histori perjalanan. Session dapat ikut terhapus bersama akun; referensi aktor audit dapat menjadi null.
- Support, notifikasi, pembayaran non-tunai, dan lokasi driver persisten belum termasuk schema MVP.
