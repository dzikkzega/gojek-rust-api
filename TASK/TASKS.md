# GOJEK-rust-api — Task Cards

Task backlog MVP berdasarkan [PRD](../PRD.md). Semua kartu dimulai dengan status **Todo**; implementasi belum dimulai.

| User Story | Fokus |
|---|---|
| [US 01 — API Login](US%2001%20-%20API%20LOGIN/US.md) | OTP, sesi, refresh, logout |
| [US 02 — Profil Penumpang](US%2002%20-%20PROFIL%20PENUMPANG/US.md) | Baca dan ubah profil |
| [US 03 — Pendaftaran Driver dan Kendaraan](US%2003%20-%20PENDAFTARAN%20DRIVER%20DAN%20KENDARAAN/US.md) | Aplikasi driver, dokumen, kendaraan, approval |
| [US 04 — Status dan Lokasi Driver](US%2004%20-%20STATUS%20DAN%20LOKASI%20DRIVER/US.md) | Availability dan lokasi terkini |
| [US 05 — Layanan dan Estimasi Tarif](US%2005%20-%20LAYANAN%20DAN%20ESTIMASI%20TARIF/US.md) | Katalog, area, quote |
| [US 06 — Pemesanan dan Matching Driver](US%2006%20-%20PEMESANAN%20DAN%20MATCHING%20DRIVER/US.md) | Booking idempotent dan klaim atomik |
| [US 07 — Status dan Tracking Perjalanan](US%2007%20-%20STATUS%20DAN%20TRACKING%20PERJALANAN/US.md) | State machine, kode mulai, tracking |
| [US 08 — Riwayat dan Pembayaran Tunai](US%2008%20-%20RIWAYAT%20DAN%20PEMBAYARAN%20TUNAI/US.md) | Riwayat dan pencatatan cash |
| [US 09 — Rating Perjalanan](US%2009%20-%20RATING%20PERJALANAN/US.md) | Rating setelah perjalanan |
| [US 10 — Admin dan Operasional](US%2010%20-%20ADMIN%20DAN%20OPERASIONAL/US.md) | Admin P0, health, readiness, tracing |

Setiap folder `US NN - ...` berisi `US.md` sebagai kartu induk dan folder `AC/` dengan kartu-kartu acceptance criteria bernomor.