# Alur booking dan matching driver

Dokumen ini menjelaskan state flow P0. Nilai operasional yang belum disepakati (radius, timeout, freshness, batas retry) tetap TODO; jangan menganggap contoh angka sebagai keputusan produk.

## Urutan happy path

1. **Quote** — penumpang mengirim pickup, destination, dan tipe layanan ke `POST /api/v1/ride-quotes`. API memvalidasi koordinat/area PostGIS, mengambil versi fare rule aktif, lalu menyimpan quote immutable beserta jarak, durasi, estimasi, mata uang, dan expiry.
2. **Create trip** — penumpang mengirim `quote_id` ke `POST /api/v1/trips` dengan `Idempotency-Key`. Dalam transaksi PostgreSQL, API mengunci/mengubah quote `active → consumed`, menyalin snapshot quote ke trip, membuat trip `searching`, dan menulis `trip_events` awal.
3. **Find candidates** — service matching mengambil driver `online`, approved, vehicle aktif dengan tipe yang sesuai, dan lokasi Redis yang masih fresh. Batas area/radius, urutan fan-out, dan freshness threshold adalah konfigurasi TODO.
4. **Offer** — tawaran dialokasikan kepada driver kandidat menurut strategi MVP yang diputuskan (serial atau batch). Per-driver offer/lease harus ber-lease/TTL; kedaluwarsa atau decline tidak mengubah kepemilikan trip.
5. **Accept atomically** — `POST /api/v1/driver/trips/{trip_id}/accept` mencoba conditional update dalam transaksi PostgreSQL hanya bila trip masih `searching` dan driver masih eligible. Tepat satu transaksi boleh memenangkan klaim. Yang lain menerima `409`.
6. **Driver arrives** — driver yang ditugaskan memanggil `/arrive`; state berubah `driver_assigned → driver_arrived` dan event dicatat.
7. **Start** — driver mengirim start code dari penumpang. Server memverifikasi hash/kode sementara, expiry dan attempt limit; bila benar state berubah `driver_arrived → in_progress`. Kode mentah tidak disimpan/log.
8. **Track** — driver mengirim koordinat ke `/driver/location`; API menyimpan posisi terkini di Redis dengan TTL. Tracking mengembalikan posisi hanya untuk peserta trip dan boleh menghilangkan lokasi yang stale.
9. **Complete** — driver memanggil `/complete`; server menentukan nilai akhir dari aturan produk yang disetujui (rumus belum diputuskan), mengubah state `in_progress → completed`, menulis event, lalu membuat/memperbarui payment cash dalam transaksi.
10. **Rate** — peserta dapat memberi paling banyak satu rating per arah setelah trip selesai.

## State transition table

| Current | Action | Next | Actor / guard | Failure |
|---|---|---|---|---|
| — | create from active, unexpired quote | `searching` | quote owner; idempotency key valid | quote expired/consumed: `422`; same key/different payload: `409` |
| `searching` | accept offer | `driver_assigned` | offered eligible driver; atomic DB claim | concurrent winner/state changed: `409` |
| `searching` | all offers time out/no candidate | `no_driver_found` | matching service | duplicate terminal update is idempotent |
| `searching` | cancel | `cancelled` | rider; cancellation policy TBD | race with accept: one transaction wins |
| `driver_assigned` | arrive | `driver_arrived` | assigned driver | wrong actor/state: `403`/`409` |
| `driver_assigned` or `driver_arrived` | cancel | `cancelled` | authorized participant; fee policy TBD | no cancel after terminal state |
| `driver_arrived` | start with valid code | `in_progress` | assigned driver; code valid/unexpired | invalid/expired/attempt limit: `409` or `422` per final contract |
| `in_progress` | complete | `completed` | assigned driver; fare computation succeeds | wrong state: `409`; dependency failure: safe `5xx` |

Only the application command that performs the transition may update `trips.status`; consumers do not write status directly. Every successful transition appends one `trip_events` row in the same DB transaction.

## Concurrency and idempotency requirements

- `POST /trips`: persist `(rider_id, idempotency_key)` and a canonical request hash/result. Same key + same payload returns the original trip; same key + different payload returns `409`. Key retention duration and canonicalization algorithm are TODO.
- Quote consumption: enforce one trip per quote with unique FK and conditional `UPDATE ... WHERE status='active' AND expires_at > now() RETURNING ...` inside the booking transaction.
- Driver claim: use a conditional write/row lock in PostgreSQL. Redis locks/leases can optimize offers but are not the correctness boundary.
- State transition + event + payment changes must commit or roll back together.
- Retry after timeout must inspect durable idempotency/transition state rather than create a second trip/event/payment.
- Location writes are replaceable ephemeral state; reject future timestamps beyond allowed clock skew (threshold TBD), out-of-range coordinates, and stale samples.

## Failure/recovery behavior

- **Redis unavailable:** do not invent driver positions. Matching may pause/fail closed; existing PostgreSQL trip state remains intact. Readiness reflects configured Redis dependency.
- **PostgreSQL unavailable:** do not accept booking/claim/state transition; never report success based only on Redis.
- **No candidates:** end search as `no_driver_found`; do not leave a permanently searching trip.
- **Driver goes stale/offline after offer:** invalidate its offer; matching can continue while trip remains `searching`.
- **Driver accept races with rider cancel:** perform both as compare-and-set state transitions. Exactly one succeeds; the loser gets `409` and reads current state.
- **Fare decision unavailable:** do not complete with a guessed amount. Keep trip in progress and surface a retryable operational error.

## Decisions still required before production

See [`../decisions/open-decisions.md`](../decisions/open-decisions.md): candidate radius/order, offer TTL/fan-out, location freshness, start-code generation/TTL/attempts, fare calculation/final fare, cancellation fee and timeout values.
