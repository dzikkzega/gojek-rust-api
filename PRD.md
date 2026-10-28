# PRD — GOJEK Rust API

**Status:** Draft v1.0  
**Produk:** Backend API untuk layanan pemesanan transportasi motor dan mobil  
**Tanggal:** 23 Oktober 2026

> Nama folder mengikuti permintaan. Dokumen ini mendefinisikan backend untuk produk ride-hailing independen; tidak menyatakan afiliasi atau integrasi resmi dengan Gojek.

## 1. Ringkasan

Bangun backend API yang memungkinkan penumpang meminta perjalanan, sistem mencarikan driver, driver menjalankan perjalanan, dan kedua pihak melihat status serta lokasi perjalanan. API menjadi sumber kebenaran untuk akun, perjalanan, tarif, dan pembayaran yang tercatat. PostgreSQL menyimpan data permanen; Redis menyimpan data sementara dan lokasi driver terkini.

Rilis pertama adalah **modular monolith**: satu aplikasi API dengan modul domain terpisah. Tidak ada microservices pada MVP.

## 2. Masalah dan tujuan

### Masalah

Penumpang membutuhkan cara memesan perjalanan dan mengetahui status driver. Driver membutuhkan cara mengatur ketersediaan, menerima pesanan, dan memperbarui status perjalanan. Operator membutuhkan sarana untuk memeriksa perjalanan dan menangani pendaftaran atau laporan.

### Tujuan produk

1. Menyediakan alur pemesanan motor/mobil dari estimasi hingga perjalanan selesai atau dibatalkan.
2. Memasangkan satu perjalanan dengan paling banyak satu driver.
3. Menjaga transisi status dan pencatatan pembayaran tetap konsisten.
4. Membatasi data akun, lokasi, dan tindakan operasional berdasarkan otorisasi.
5. Menyediakan API yang dapat diuji, dipantau, dan dikembangkan tanpa memecah layanan terlalu dini.

### Bukan tujuan MVP

- GoFood, kurir/pengiriman barang, dan layanan non-transportasi.
- Dompet digital, saldo, top-up, promo, loyalty, langganan.
- Tarif dinamis kompleks, multi-stop, pemesanan terjadwal, perjalanan antar-kota.
- Aplikasi mobile/web atau antarmuka admin.
- Microservices, event broker, atau infrastruktur yang tidak diperlukan untuk menguji alur inti.

## 3. Pengguna dan akses

- **Penumpang:** membuat estimasi dan pesanan, mengikuti perjalanan, membayar sesuai metode, memberi rating, mengelola profil.
- **Driver:** mendaftar, mengelola kendaraan, online/offline, mengirim lokasi, menerima dan menjalankan perjalanan.
- **Admin/CS:** meninjau pendaftaran, memeriksa perjalanan, menangani tiket, dan melakukan tindakan yang tercatat di audit log.
- **Penyedia pembayaran:** hanya memanggil webhook jika pembayaran non-tunai ditambahkan pada fase berikutnya.

Semua resource milik pengguna hanya dapat diakses pemiliknya, peserta perjalanan, atau staf berizin sesuai fungsi. Role admin tidak otomatis memberi akses tanpa batas; tindakan sensitif dicatat.

## 4. Pilihan produk yang ditetapkan

| Area | Keputusan v1 |
|---|---|
| Produk | API backend ride-hailing motor dan mobil |
| Bentuk layanan | REST API JSON; WebSocket untuk update langsung perjalanan bila termasuk MVP teknis |
| Prefix | `/api/v1` |
| Auth awal | Email terverifikasi + OTP; SMTP relay/provider produksi TBD, Mailpit untuk development lokal |
| Pembayaran awal | Tunai; pembayaran non-tunai ditunda |
| Status perjalanan | State machine eksplisit, perubahan hanya melalui aksi yang sah |
| Arsitektur | Modular monolith |
| Data permanen | PostgreSQL + PostGIS |
| Lokasi live/cache | Redis; bukan sumber kebenaran transaksi |
| Framework/runtime | Axum + Tokio |
| Database access/JSON | SQLx + Serde |
| Logging | Tracing; tidak mencatat OTP, token, kredensial, atau koordinat presisi ke log umum |

### Keputusan yang tetap perlu diisi sebelum implementasi produksi

- Wilayah/kota peluncuran, batas area layanan, mata uang, dan bahasa.
- Rumus tarif, minimum tarif, pembulatan, biaya pembatalan, dan siapa yang menanggungnya.
- Provider peta/rute/geocoding dan email SMTP relay produksi (local development uses Mailpit).
- Usia minimum, syarat verifikasi identitas, retensi dokumen, serta kebijakan privasi.
- SLA, kapasitas puncak, target ketersediaan, dan target latensi numerik.
- Payment gateway untuk fase non-tunai.

Nilai tersebut harus menjadi konfigurasi atau keputusan produk; jangan mengarang nilai produksi saat implementasi.

## 5. Ruang lingkup rilis

### P0 — MVP

- OTP, sesi, profil dasar.
- Pendaftaran driver dan kendaraan; persetujuan admin.
- Driver online/offline dan pembaruan lokasi.
- Tipe layanan, area, dan estimasi tarif.
- Buat/cari/terima pesanan, pembatalan, penjemputan, mulai, selesai.
- Tracking dan riwayat perjalanan.
- Tunai sebagai metode pembayaran; catat kewajiban/status pembayaran.
- Rating setelah perjalanan.
- Health/readiness endpoint, log terstruktur, audit tindakan admin, pengujian.

### P1 — setelah MVP tervalidasi

- Metode pembayaran non-tunai dan webhook provider.
- Push notification/device token, tiket bantuan dan percakapan CS.
- Promo, tarif dinamis, pembayaran/refund otomatis.
- Fitur operasional lanjutan dan optimasi matching.

## 6. Alur utama

1. Pengguna meminta OTP dan memverifikasinya; server membuat sesi dan mengembalikan token.
2. Penumpang mengirim titik jemput, tujuan, serta tipe layanan untuk meminta quote.
3. Quote menyertakan estimasi harga/jarak/durasi, masa berlaku, dan identitas quote. Estimasi bukan harga final jika aturan produk mengizinkan penyesuaian.
4. Penumpang membuat perjalanan berdasarkan quote dengan `Idempotency-Key`.
5. Sistem mencari driver yang online, memenuhi tipe layanan/area, dan lokasinya masih segar; penawaran dikirim kepada kandidat.
6. Satu driver menerima penawaran secara atomik. Jika tidak ada yang menerima dalam batas waktu, perjalanan berakhir sebagai `no_driver_found`.
7. Driver tiba, penumpang memberikan kode mulai (OTP perjalanan), lalu driver memulai perjalanan.
8. Lokasi dan status diperbarui selama perjalanan.
9. Driver menyelesaikan perjalanan; server menetapkan total akhir dan status pembayaran tunai.
10. Penumpang memberi rating. Perjalanan muncul di riwayat kedua pihak.

## 7. Status dan aturan perjalanan

### Status

`searching → driver_assigned → driver_arrived → in_progress → completed`

Status terminal alternatif: `cancelled`, `no_driver_found`. Alasan pembatalan disimpan terpisah. Status pembayaran tidak digabung dengan status perjalanan.

### Invarian

- Hanya penumpang pemilik yang boleh membuat perjalanan atas akunnya.
- Satu perjalanan hanya punya satu driver aktif pada satu waktu.
- Penerimaan driver dan klaim pesanan harus atomik; permintaan bersamaan tidak boleh menghasilkan dua pemenang.
- Hanya aksi valid dari status saat ini yang diterima; retry tidak menggandakan efek.
- Mulai perjalanan memerlukan verifikasi kode mulai, dengan batas percobaan dan masa berlaku.
- Pembatalan mengikuti kebijakan yang ditetapkan; endpoint tidak boleh mengubah perjalanan selesai menjadi batal.
- `completed_at`, `cancelled_at`, dan total akhir konsisten dengan status.
- PostgreSQL menjadi sumber kebenaran transaksi. Data Redis dapat dibangun ulang atau kedaluwarsa tanpa menghilangkan riwayat transaksi.

## 8. API v1

Semua endpoint JSON menggunakan prefix `/api/v1`, kecuali webhook dan operasional internal sesuai kebutuhan deployment. Endpoint bertanda **P0** termasuk MVP. Endpoint lain adalah P1 atau dukungan operasional yang dapat dibatasi jaringan.

### 8.1 Auth dan akun

| Method | Path | Akses | Prioritas / perilaku |
|---|---|---|---|
| `POST` | `/auth/otp/request` | Publik | P0; meminta OTP email, rate-limited; respons tidak membocorkan apakah akun sudah terdaftar |
| `POST` | `/auth/otp/verify` | Publik | P0; verifikasi OTP, buat akun penumpang bila belum ada, buat sesi |
| `POST` | `/auth/token/refresh` | Sesi | P0; rotasi/refresh token dan cabut token lama sesuai desain sesi |
| `POST` | `/auth/logout` | Login | P0; cabut sesi saat ini |
| `POST` | `/auth/logout-all` | Login | P1; cabut seluruh sesi pengguna |
| `GET` | `/auth/sessions` | Login | P1; daftar sesi milik sendiri, tanpa rahasia token |
| `DELETE` | `/auth/sessions/{session_id}` | Pemilik | P1; cabut sesi tertentu |
| `GET` | `/me` | Login | P0; profil dan role/status |
| `PATCH` | `/me` | Login | P0; ubah field profil yang diizinkan |
| `DELETE` | `/me` | Login | P1; ajukan penghapusan/nonaktifkan akun sesuai kebijakan retensi |

### 8.2 Alamat dan favorit

| Method | Path | Akses | Prioritas |
|---|---|---|---|
| `GET` | `/me/addresses` | Penumpang | P1 |
| `POST` | `/me/addresses` | Penumpang | P1 |
| `PATCH` | `/me/addresses/{address_id}` | Pemilik | P1 |
| `DELETE` | `/me/addresses/{address_id}` | Pemilik | P1 |
| `GET` | `/me/favorites` | Penumpang | P1 |
| `POST` | `/me/favorites` | Penumpang | P1 |
| `DELETE` | `/me/favorites/{favorite_id}` | Pemilik | P1 |

### 8.3 Driver, aplikasi, dan kendaraan

| Method | Path | Akses | Prioritas / perilaku |
|---|---|---|---|
| `POST` | `/driver/applications` | Login | P0; ajukan pendaftaran |
| `GET` | `/driver/application` | Driver | P0; lihat status aplikasi sendiri |
| `POST` | `/driver/application/documents` | Driver | P0; unggah dokumen melalui storage/provider yang ditentukan sebelum implementasi |
| `POST` | `/driver/application/submit` | Driver | P0; kirim untuk ditinjau |
| `GET` | `/driver/me` | Driver | P0; profil dan status kelayakan |
| `PATCH` | `/driver/me` | Driver | P1; ubah field yang diizinkan |
| `GET` | `/driver/vehicles` | Driver | P0 |
| `POST` | `/driver/vehicles` | Driver | P0 |
| `PATCH` | `/driver/vehicles/{vehicle_id}` | Pemilik | P1 |
| `DELETE` | `/driver/vehicles/{vehicle_id}` | Pemilik | P1 |
| `PUT` | `/driver/availability` | Driver disetujui | P0; ubah online/offline |
| `PUT` | `/driver/location` | Driver online | P0; kirim koordinat dan waktu observasi; validasi rentang/freshness/frekuensi |
| `GET` | `/driver/trips/offers` | Driver online | P0; tawaran aktif milik driver |

### 8.4 Layanan, area, dan quote

| Method | Path | Akses | Prioritas |
|---|---|---|---|
| `GET` | `/ride-types` | Publik | P0; tipe layanan aktif dan atribut yang boleh ditampilkan |
| `POST` | `/ride-quotes` | Penumpang | P0; asal, tujuan, tipe layanan; validasi area; hasil estimasi dan masa berlaku |
| `GET` | `/ride-quotes/{quote_id}` | Pemilik | P0; baca quote milik sendiri selama masih tersedia |

### 8.5 Perjalanan

| Method | Path | Akses | Prioritas / perilaku |
|---|---|---|---|
| `POST` | `/trips` | Penumpang | P0; menerima quote valid; wajib `Idempotency-Key` |
| `GET` | `/trips/current` | Peserta | P0; perjalanan aktif saat ini |
| `GET` | `/trips` | Peserta | P0; riwayat dengan cursor pagination dan filter terbatas |
| `GET` | `/trips/{trip_id}` | Peserta/Admin berizin | P0; detail dan status |
| `POST` | `/trips/{trip_id}/cancel` | Peserta | P0; alasan dan hasil sesuai kebijakan pembatalan |
| `POST` | `/driver/trips/{trip_id}/accept` | Driver yang ditawari | P0; klaim atomik; idempotent |
| `POST` | `/driver/trips/{trip_id}/decline` | Driver yang ditawari | P0 |
| `POST` | `/driver/trips/{trip_id}/arrive` | Driver yang ditugaskan | P0 |
| `POST` | `/driver/trips/{trip_id}/start` | Driver yang ditugaskan | P0; memverifikasi kode mulai |
| `POST` | `/driver/trips/{trip_id}/complete` | Driver yang menjalankan perjalanan | P0; simpan total akhir dan waktu selesai |
| `GET` | `/trips/{trip_id}/tracking` | Peserta | P0; lokasi/status terkini yang boleh dilihat |
| `GET` | `/realtime` | Login, otorisasi per perjalanan | P1; WebSocket untuk event update status/lokasi |

Event realtime yang direncanakan: `trip.status_changed`, `trip.driver_assigned`, `driver.location_updated`. Server hanya mengirim event kepada peserta yang berhak. Detail format event dan reconnect/cursor ditetapkan dalam spesifikasi API sebelum implementasi.

### 8.6 Pembayaran dan rating

| Method | Path | Akses | Prioritas / perilaku |
|---|---|---|---|
| `GET` | `/payment-methods` | Login | P0; metode tunai pada MVP |
| `GET` | `/trips/{trip_id}/payment` | Peserta | P0; nominal, metode, status, tanpa data sensitif |
| `GET` | `/me/payment-methods` | Penumpang | P1; metode tersimpan non-tunai |
| `POST` | `/me/payment-methods/setup` | Penumpang | P1; memulai setup melalui provider |
| `DELETE` | `/me/payment-methods/{method_id}` | Pemilik | P1 |
| `POST` | `/webhooks/payments/{provider}` | Provider terverifikasi | P1; verifikasi signature, idempotent, jangan percaya status dari client |
| `POST` | `/trips/{trip_id}/rating` | Peserta, setelah selesai | P0; satu rating per pemberi rating/perjalanan |

### 8.7 Notifikasi dan bantuan

| Method | Path | Akses | Prioritas |
|---|---|---|---|
| `GET` | `/me/notifications` | Login | P1 |
| `PATCH` | `/me/notifications/{notification_id}` | Pemilik | P1; tandai dibaca |
| `POST` | `/me/devices` | Login | P1; daftarkan token push |
| `DELETE` | `/me/devices/{device_id}` | Pemilik | P1 |
| `POST` | `/trips/{trip_id}/reports` | Peserta | P1 |
| `POST` | `/support/tickets` | Login | P1 |
| `GET` | `/support/tickets` | Login | P1; hanya tiket sendiri kecuali CS |
| `GET` | `/support/tickets/{ticket_id}` | Pemilik/CS | P1 |
| `POST` | `/support/tickets/{ticket_id}/messages` | Pemilik/CS | P1 |

### 8.8 Admin dan operasional

| Method | Path | Akses | Prioritas |
|---|---|---|---|
| `GET` | `/admin/driver-applications` | Admin | P0; filter status dan pagination |
| `GET` | `/admin/driver-applications/{application_id}` | Admin | P0 |
| `POST` | `/admin/driver-applications/{application_id}/approve` | Admin berizin | P0; audit log |
| `POST` | `/admin/driver-applications/{application_id}/reject` | Admin berizin | P0; alasan wajib; audit log |
| `GET` | `/admin/users` | Admin/CS sesuai izin | P1; pencarian dengan pembatasan data pribadi |
| `GET` | `/admin/users/{user_id}` | Admin/CS sesuai izin | P1 |
| `PATCH` | `/admin/users/{user_id}/status` | Admin berizin | P1; alasan dan audit log |
| `GET` | `/admin/trips` | Admin/CS | P0; filter/pagination |
| `GET` | `/admin/trips/{trip_id}` | Admin/CS | P0; akses diaudit |
| `GET` | `/admin/service-areas` | Admin | P0 |
| `POST` | `/admin/service-areas` | Admin berizin | P0; perubahan diaudit |
| `PATCH` | `/admin/service-areas/{area_id}` | Admin berizin | P0; perubahan diaudit |
| `GET` | `/admin/fare-rules` | Admin | P0 |
| `POST` | `/admin/fare-rules` | Admin berizin | P0; versi berlaku dan audit log |
| `PATCH` | `/admin/fare-rules/{rule_id}` | Admin berizin | P0; jangan mengubah quote/trip lama secara retroaktif |
| `GET` | `/admin/audit-logs` | Admin berizin | P1; read-only dan terbatas |
| `GET` | `/health` | Infrastruktur | P0; proses hidup, tanpa rahasia |
| `GET` | `/ready` | Infrastruktur | P0; siap menerima trafik berdasarkan dependency wajib |
| `GET` | `/metrics` | Internal/terproteksi | P1; jangan dipublikasikan tanpa kontrol akses |

## 9. Kontrak API

- JSON UTF-8; waktu dalam UTC ISO 8601; ID resource tidak berurutan secara mudah ditebak.
- Semua list memakai pagination berbatas; default dan batas maksimum ditetapkan sebelum implementasi.
- Respons error konsisten, misalnya `{ "error": { "code": "...", "message": "...", "request_id": "..." } }`; jangan mengirim stack trace atau detail internal.
- Validasi request dilakukan di boundary API: enum/status yang dikenal, koordinat dalam rentang, ukuran payload, panjang string, pagination, serta kepemilikan resource.
- `POST /trips` dan aksi yang berdampak finansial/status mendukung idempotency. Perilaku key, masa simpan, dan konflik payload didokumentasikan.
- Gunakan kode status HTTP konsisten: `400` input tidak valid, `401` belum terautentikasi, `403` tidak berhak, `404` tidak ditemukan/tidak dapat diakses, `409` konflik state/idempotency, `422` aturan domain tidak terpenuhi, `429` rate limit, `500` error tak terduga.
- Versi API berada di path `/api/v1`; perubahan breaking memerlukan versi baru atau kebijakan kompatibilitas yang terdokumentasi.

## 10. Data inti

Entitas konseptual (nama kolom final ditetapkan dalam skema/migrasi):

- `users`: identitas akun berbasis email ternormalisasi/terverifikasi, nomor telepon opsional sebagai kontak, profil, status, timestamps.
- `auth_sessions`: sesi/token yang dapat dicabut, masa berlaku, metadata perangkat minimum.
- `driver_profiles`: status pendaftaran, status operasional, rating agregat yang dapat dihitung ulang.
- `driver_documents`: metadata dan status review; berkas tidak disimpan sebagai teks di database/log.
- `vehicles`: pemilik, tipe layanan, nomor registrasi dan atribut kendaraan sesuai kebutuhan.
- `service_areas`: batas geografis dan status area.
- `fare_rules`: aturan tarif berversi dan masa berlaku.
- `ride_quotes`: titik asal/tujuan, estimasi, aturan tarif yang dipakai, expiry.
- `trips`: penumpang, driver, tipe, titik, status, harga, waktu, alasan batal, idempotency reference.
- `trip_events`: riwayat perubahan status yang append-only untuk audit domain.
- `payments`: metode, nominal, status, provider reference bila kelak non-tunai.
- `ratings`: perjalanan, pemberi, penerima, nilai, komentar opsional dengan batas panjang.
- `support_tickets`, `notifications`, `audit_logs`: P1 kecuali audit log tindakan admin yang diperlukan sejak P0.

PostGIS dipakai untuk data geospasial persisten yang diperlukan, termasuk area layanan dan titik perjalanan. Redis menyimpan koordinat/freshness driver dan data sementara; TTL serta strategi kedaluwarsa wajib. Jangan menjadikan Redis satu-satunya tempat menyimpan status order.

## 11. Arsitektur dan stack

### Stack yang disetujui

- **Axum + Tokio:** routing HTTP, handler, async runtime.
- **PostgreSQL + PostGIS:** data permanen, transaksi, constraint, dan query spasial.
- **Redis:** lokasi driver terkini, cache, pencarian kandidat dekat, serta data sementara yang aman untuk kedaluwarsa.
- **SQLx + Serde:** akses database asynchronous, migrasi/query mapping, serialisasi JSON.
- **Tracing:** structured logs, spans, request correlation, dan diagnostic context.

Tidak menambahkan framework/library/platform lain ke keputusan stack PRD ini. Provider eksternal yang diperlukan (OTP, peta, payment, push, penyimpanan dokumen) belum dipilih dan harus menjadi keputusan terpisah sebelum fitur terkait dibangun.

### Modul logis dalam satu aplikasi

`auth`, `users`, `drivers`, `vehicles`, `catalog`, `quotes`, `matching`, `trips`, `payments`, `ratings`, `support`, `admin`, `health`.

Alur request: route/ekstraksi/validasi → aturan domain → akses repository/SQLx dan Redis sesuai fungsi → respons/error terstandar. Handler tidak memuat semua aturan bisnis; transaksi perjalanan tetap berada pada PostgreSQL.

## 12. Keamanan dan privasi

- TLS diwajibkan di deployment; rahasia hanya lewat secret/environment management deployment, bukan repository atau log.
- OTP harus memiliki masa berlaku, batas percobaan, rate limit per email/IP, dan penyimpanan aman; respons request tidak mengungkap keberadaan akun.
- Token sesi harus dapat dicabut dan tidak pernah ditampilkan kembali di log. Pilihan format token/provider masih ditetapkan saat desain auth teknis.
- Otorisasi server-side pada setiap resource; jangan percaya `user_id`, role, tarif, status, maupun jumlah pembayaran dari client.
- Driver hanya dapat mengubah perjalanan yang ditugaskan kepadanya; penumpang hanya dapat membaca perjalanan miliknya.
- Koordinat lokasi adalah data sensitif: kirim hanya kepada peserta yang sah, batasi retensi dan akses, kurangi presisi pada log, serta tetapkan masa hidup di Redis.
- Validasi file/dokumen, ukuran, tipe konten, dan akses; jangan menganggap nama file dari client tepercaya.
- Webhook P1 diverifikasi tanda tangan, timestamp/replay policy, dan idempotency.
- Admin menggunakan least privilege; perubahan tarif/status, persetujuan dokumen, dan akses detail perjalanan tercatat di audit log.
- Rate limit untuk OTP, login, quote, lokasi, pembatalan, dan endpoint publik.
- CORS, batas request body, timeout, dan header keamanan ditentukan untuk domain client sebelum deployment publik.
- Kebijakan privasi, persetujuan lokasi, retensi/hapus akun, dan penanganan permintaan data harus ditetapkan sebelum produksi.

## 13. Keandalan dan konsistensi

- Constraint database menjaga foreign key, status yang valid, keunikan yang diperlukan, dan integritas transaksi.
- Operasi matching/acceptance menggunakan transaksi/conditional update agar tidak ada double assignment.
- Redis dianggap cache/ephemeral. Jika Redis pulih atau data kedaluwarsa, API tidak boleh merusak data perjalanan PostgreSQL; driver mungkin perlu mengirim ulang lokasinya.
- Kegagalan dependency dibedakan dari input salah; jangan mengubah error database menjadi sukses semu.
- Endpoint aksi memiliki timeout dan perilaku retry yang terdokumentasi; operasi non-idempotent tidak boleh diulang tanpa key.
- Backup dan pemulihan PostgreSQL harus disiapkan sebelum layanan menyimpan data pengguna nyata.

## 14. Observability

- Tracing mencakup request ID, route, status, durasi, dependency, dan error class; hindari payload pribadi.
- Health membedakan liveness (`/health`) dari readiness (`/ready`). Jangan mengekspos detail koneksi atau secret.
- Metrik minimum: jumlah/latensi/status request, error, koneksi DB, kegagalan Redis, quote-to-booking, waktu pencarian driver, perjalanan per status, dan pembatalan.
- Log/audit admin terpisah dari log aplikasi biasa; aksesnya dibatasi.
- Nilai SLO dan target retensi log ditetapkan sebelum produksi.

## 15. Pengujian dan kriteria penerimaan

### Pengujian wajib

- Unit test aturan tarif, izin, transisi status, expiry quote/OTP, dan pembatalan.
- Integration test PostgreSQL/PostGIS dan Redis untuk query spasial, lokasi stale, transaksi, serta fallback kegagalan.
- API test untuk status HTTP, validasi, error schema, auth, dan akses lintas pengguna.
- Uji konkurensi: dua driver menerima pesanan yang sama; hanya satu berhasil.
- Uji idempotency: request pemesanan berulang dengan key sama menghasilkan satu perjalanan; key sama dengan payload berbeda menghasilkan konflik.
- Uji keamanan: enumerasi email, brute-force OTP, akses resource ID milik orang lain, input lokasi tidak valid, dan role escalation.

### MVP diterima jika

1. Penumpang dapat OTP login, meminta quote, memesan, melihat status, dan melihat riwayat.
2. Driver yang disetujui dapat online, memperbarui lokasi, menerima maksimal satu tawaran, memverifikasi kode mulai, dan menyelesaikan perjalanan.
3. Sistem menolak transisi status ilegal dan mencegah double assignment pada request bersamaan.
4. Perjalanan dan pembayaran tunai tercatat konsisten di PostgreSQL, meskipun Redis mengalami restart.
5. Hanya pemilik/peserta/staf berizin dapat mengakses resource terkait.
6. Endpoint admin P0 dapat mengelola aplikasi driver, area, aturan tarif, dan pemeriksaan perjalanan; tindakan penting masuk audit log.
7. `/health` dan `/ready` berperilaku sesuai definisi; log tidak memuat token, OTP, atau koordinat presisi.
8. Pengujian kritis berjalan otomatis dan lulus sebelum rilis.

## 16. Risiko dan mitigasi

| Risiko | Mitigasi |
|---|---|
| Matching driver terlalu rumit | Mulai dari radius, status online, freshness, dan tipe layanan; optimasi setelah data nyata tersedia |
| Dua driver menerima pesanan | Klaim atomik/constraint dan tes konkurensi |
| Lokasi palsu atau usang | Validasi koordinat/waktu, freshness threshold, monitoring anomali; kebijakan anti-fraud menyusul |
| Tarif tidak akurat | Quote berversi, masa berlaku, provider rute dan aturan tarif diputuskan sebelum rilis |
| OTP mahal atau disalahgunakan | Gunakan SMTP + Mailpit untuk lokal; pilih relay produksi, rate limit, cooldown, dan limit per email/IP |
| Redis hilang | Simpan transaksi di PostgreSQL; data lokasi diminta ulang dari driver |
| Scope membesar menyerupai super-app | Tolak layanan non-transportasi dari MVP; kembangkan hanya berdasar validasi produk |
| Data lokasi/dokumen sensitif | Least privilege, retensi terbatas, audit, redaksi log, kebijakan privasi sebelum produksi |

## 17. Urutan implementasi yang disarankan

1. Fondasi aplikasi, konfigurasi, tracing, health/readiness, skema dan migrasi.
2. Auth OTP dan profil dengan provider OTP sebagai dependency yang harus dipilih.
3. Pendaftaran driver, kendaraan, persetujuan admin.
4. Area layanan, tipe ride, aturan tarif, quote.
5. Ketersediaan dan lokasi driver; pencarian kandidat.
6. State machine perjalanan, pemesanan idempotent, klaim atomik.
7. Tracking, kode mulai, penyelesaian, tunai, riwayat, rating.
8. Hardening keamanan, tes konkurensi/integrasi, backup, readiness produksi.
9. Setelah stabil: payment gateway, notifikasi, support, promo, fitur tambahan.

## 18. Glosarium

- **Quote:** estimasi perjalanan dan tarif dengan masa berlaku.
- **Matching:** proses memilih dan menawarkan perjalanan kepada driver yang memenuhi syarat.
- **Freshness:** usia maksimum pembaruan lokasi agar dianggap terkini.
- **Idempotency:** jaminan pengulangan request yang sama tidak menggandakan efek.
- **Modular monolith:** satu deployment aplikasi dengan batas modul internal yang jelas.
