# SPRINT ROADMAP 7 HARI :: HERMES AGENT FOUNDRY
**Arsitektur & Hak Cipta:** Bagas Cihuy (Bagas Saputra)  
**Jendela Eksekusi:** Pukul 02:00 - 04:00 WIB (Setiap Malam)  
**Target Selesai:** 7 Hari Kalender  
**Status Eksekusi:** Hari 1, Hari 2 & Hari 3 Selesai Dituntaskan (Hari 4 Siap)  

---

## DAFTAR PEMBAGIAN TUGAS PER HARI

### HARI 1: SQLite Ledger, Ephemeral Worker Engine & Event Loop (Malam Ke-1)
- **Fokus Utama:** Menyempurnakan penyimpanan basis data lokal dan runner eksekusi proses sementara.
- **Tugas Koding:**
  - [x] Rancang skema database `foundry.sqlite` (tabel `agents`, `execution_runs`).
  - [x] Implementasikan `src/engine/ephemeral_worker.rs` untuk spawn isolasi tugas berbasis thread/process.
  - [x] Bangun event loop transisi state dari simpul ke simpul berikutnya.
  - [x] Uji performa latensi alokasi memori (cold-start < 10ms, RAM < 15MB).
- **Misi Belajar & Inovasi:**
  - Merekam pola efisiensi *Zero-Copy Deserialization* pada Rust Serde.
  - Dokumentasikan catatan pengalaman ke vault: `EXPERIENCES/EXP-FOUNDRY-HARI-1.md`.

---

### HARI 2: Adapter Modul Core: Triggers, Context & KV Memory (Malam Ke-2)
- **Fokus Utama:** Menghubungkan pemicu dan lapisan memori agen.
- **Tugas Koding:**
  - [x] Buat handler `modules/triggers/cron.rs` (bridge ke Hermes cron ticker).
  - [x] Buat handler `modules/triggers/webhook.rs` (inbound HTTP POST listener).
  - [x] Buat handler `modules/context/vault_reader.rs` (pembaca terisolasi ke Obsidian Vault).
  - [x] Buat handler `modules/context/kv_store.rs` (state persisten antar-run).
- **Misi Belajar & Inovasi:**
  - Pola isolasi direktori (*chroot sandbox*) untuk mencegah agen mengakses file sensitif.
  - Dokumentasikan catatan pengalaman ke vault: `EXPERIENCES/EXP-FOUNDRY-HARI-2.md`.

---

### HARI 3: Cognitive Reasoning & Hermes Model Bridge (Malam Ke-3)
- **Fokus Utama:** Mengintegrasikan pemanggil model AI cerdas (Claude Sonnet 4.6, GPT-6 Astra via 9Router).
- **Tugas Koding:**
  - [x] Implementasikan `src/engine/llm_client.rs` dengan streaming SSE dan connection pooling.
  - [x] Buat modul `modules/reasoning/react.rs` (Thought-Action-Observation loop).
  - [x] Buat modul `modules/reasoning/critic.rs` (Double-check guardrail sebelum eksekusi).
  - [x] Integrasikan `CircuitBreaker` untuk memutus loop reasoning jika melampaui token budget.
- **Misi Belajar & Inovasi:**
  - Pola *Dynamic Temperature Decay* untuk meminimalkan halusinasi secara matematis.
  - Dokumentasikan catatan pengalaman ke vault: `EXPERIENCES/EXP-FOUNDRY-HARI-3.md`.

---

### HARI 4: Toolkits & Execution Sandboxing (Malam Ke-4)
- **Fokus Utama:** Membekali agen dengan perkakas kerja lapangan yang aman.
- **Tugas Koding:**
  - [ ] Modul `modules/tools/web_search.rs` (pencarian fakta web real-time).
  - [ ] Modul `modules/tools/terminal_exec.rs` (eksekusi bash terisolasi dengan filter blacklist perintah destruktif).
  - [ ] Modul `modules/tools/browser_bridge.rs` (antarmuka ke Browser Use Chromium).
  - [ ] Modul `modules/tools/http_fetch.rs` (pemanggil REST API eksternal).
- **Misi Belajar & Inovasi:**
  - Merumuskan aturan keamanan eksekusi alat mandiri (*Deterministic Parameter Coercion*).
  - Dokumentasikan catatan pengalaman ke vault: `EXPERIENCES/EXP-FOUNDRY-DAY-4-TOOL-SAFETY.md`.

---

### HARI 5: No-Code Web Studio: Prompt-to-Agent & Template Picker (Malam Ke-5)
- **Fokus Utama:** Antarmuka web pengguna untuk pembuatan instan.
- **Tugas Koding:**
  - [ ] Buat antarmuka HTML/CSS/JS di `public/` dengan standar Tactile Pastel Pop (Warm Paper & Obsidian, Zero Emoji).
  - [ ] Implementasikan endpoint API `POST /api/compiler/prompt-to-agent` di Rust.
  - [ ] Hubungkan generator prompt dengan parser DAG otomatis.
  - [ ] Buat katalog visual untuk 5 template bawaan dengan tombol 1-klik deploy.
- **Misi Belajar & Inovasi:**
  - Evaluasi heuristik kompilasi teks alami menjadi topologi graf yang bebas cacat.
  - Dokumentasikan catatan pengalaman ke vault: `EXPERIENCES/EXP-FOUNDRY-DAY-5-PROMPT-COMPILER.md`.

---

### HARI 6: Visual Modular Canvas (Lego Block Builder) (Malam Ke-6)
- **Fokus Utama:** Kanvas visual drag-and-drop untuk merakit modul secara interaktif.
- **Tugas Koding:**
  - [ ] Buat modul kanvas visual interaktif berbasis SVG/HTML Canvas di `public/js/visual_canvas.js`.
  - [ ] Implementasikan sistem kabel konektor modular dengan validasi tipe data real-time.
  - [ ] Sediakan fitur live DAG validation di browser (langsung mendeteksi siklus sebelum disimpan).
  - [ ] Tambahkan dukungan ergonomi mobile penuh (Bottom Navigation Dock & Touch Gestures).
- **Misi Belajar & Inovasi:**
  - Pola rendering graf reaktif berbobot ringan tanpa dependensi pustaka frontend eksternal berat.
  - Dokumentasikan catatan pengalaman ke vault: `EXPERIENCES/EXP-FOUNDRY-DAY-6-VISUAL-CANVAS.md`.

---

### HARI 7: Multi-Agent Stress Test, Ciptakan Skill Baru & Rilis Produksi (Malam Ke-7)
- **Fokus Utama:** Pengujian ketahanan akhir, penciptaan skill Hermes, dan rilis v1.0.0.
- **Tugas Koding:**
  - [ ] Jalankan uji beban 50 siklus eksekusi agen simultan.
  - [ ] Verifikasi kuota RAM server tetap stabil $\le 9.0$ GB (target engine < 25MB RAM).
  - [ ] Ciptakan skill baru Hermes: `hermes-agent-foundry-builder` di `~/.hermes/skills/`.
  - [ ] Tag git rilis `v1.0.0-production` dan sinkronisasi push akhir ke GitHub.
  - [ ] Sajikan Laporan Rangkuman Final 7 Hari ke Mas Bagas.
- **Misi Belajar & Inovasi:**
  - Evaluasi empiris trade-off performa Rust vs Node.js untuk runtime agen otonom.
  - Dokumentasikan sintesis akhir ke vault: `SYSTEM/HERMES-FOUNDRY-PRODUCTION-LESSONS.md`.
