# DarkClaude

Chat IA cyberpunk (vert néon sur noir), prix en gourdes haïtiennes (HTG).

```
public/   frontend : landing, auth, pricing (+FAQ), contact, demo (chat), admin, pages légales
server/   backend Rust (axum + PostgreSQL + Redis), sert aussi le dossier public/
```

## 1. Prérequis
- VPS Ubuntu, Rust stable (`curl https://sh.rustup.rs -sSf | sh`), PostgreSQL, Redis, nginx.
- Clés à (re)générer vous-même : **OpenRouter** (`OPENROUTER_API_KEY`) et **Tavily** (`SEARCH_API_KEY`, commence par `tvly-`). Ne jamais les committer ni les coller dans un chat.
- Numéros NatCash / MonCash pour le paiement manuel.

## 2. Installation
```bash
sudo apt install -y postgresql redis-server nginx build-essential pkg-config
sudo -u postgres createuser darkclaude -P && sudo -u postgres createdb darkclaude_db -O darkclaude
cd server && cp .env.example .env && nano .env     # remplir les variables
cargo build --release                               # le schéma SQL s'applique tout seul au démarrage
./target/release/darkclaude-server
```
Service permanent (systemd) : `ExecStart=/chemin/server/target/release/darkclaude-server`, `WorkingDirectory=/chemin/server`, `Restart=always`, `User=` non-root.

nginx (pas de buffering, indispensable pour le streaming du chat) :
```nginx
server {
  server_name darkclaude.online;
  location / { proxy_pass http://127.0.0.1:8585; proxy_http_version 1.1; proxy_buffering off;
               proxy_read_timeout 300s; proxy_set_header Host $host;
               proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for; }
}
```
Puis `certbot --nginx -d darkclaude.online`.

## 3. Premier démarrage
1. Mets ton email dans `ADMIN_EMAILS`, crée un compte avec cet email : il devient admin (`/admin.html`).
2. Vérifie que chaque `MODEL_*` existe dans le catalogue OpenRouter (https://openrouter.ai/models). Un identifiant inexistant fait échouer le chat.
3. Paiement manuel : l'utilisateur paie par NatCash/MonCash, saisit la référence sur `/pricing.html`, tu valides dans `/admin.html`.

## 4. Plans (`server/src/plans.rs`)
Prix = euros × `FX_EUR_HTG`, arrondis à 50 HTG. Le quota se remet à zéro à minuit (heure d'Haïti ≈ UTC-5).

| Plan | HTG | Messages/jour | Modèle | Images | Web |
|---|---|---|---|---|---|
| Gratuit | 0 | 2 | `MODEL_FREE` | non | non |
| Standard (`starter`) | 1 500 /mois | 60 | `MODEL_STANDARD` | non | non |
| Premium (`unlimited`) | 3 000 /mois | 300 | `MODEL_PREMIUM` | oui | oui |
| Lifetime | 10 350, paiement unique | 1000 | `MODEL_PREMIUM` | oui | oui |

## 5. Routes API
| Route | Rôle |
|---|---|
| `GET /api/health`, `GET /api/runtime` | état (base + Redis), limites |
| `POST /api/auth/register`, `/login`, `/logout`, `GET /api/auth/me` | comptes, jeton dans l'en-tête `x-session-token` |
| `POST /api/chat` | chat en streaming NDJSON (`delta`, `ping`, `error`) ou JSON `{reply}` ; contrôle plan, quota, modération, recherche web, images |
| `GET /api/billing/plans`, `/pay-info`, `POST /manual-request`, `/cancel-subscription` | tarifs HTG, paiement manuel, résiliation |
| `POST /api/contact` | formulaire de contact |
| `GET /api/admin/users`, `/payments`, `POST /payments/:id/approve`, `/reject`, `/users/:id/plan` | administration |

## 6. Sécurité
- Mots de passe hachés (Argon2), sessions opaques stockées hachées (SHA-256), limitation de tentatives de connexion par IP, limite de 20 messages/minute.
- Le plan, le mode et le quota sont toujours revérifiés côté serveur. Le client n'est jamais cru.
- Prompt système : « Tu es DarkClaude, l'assistant de cette plateforme. » Le vrai nom du modèle n'est jamais renvoyé.
- Filtre minimal côté serveur (contenus sexuels impliquant des mineurs, armes chimiques/biologiques). À compléter par un service de modération avant une vraie ouverture au public.
- Si une clé a été exposée, révoque-la et crée-en une nouvelle.

## 7. À faire ensuite
Emails (Resend) pour reset de mot de passe, paiement automatique NatCash/MonCash, vraies mentions légales, vrais chiffres de preuve sociale. Textes légaux : à faire valider par un juriste.
