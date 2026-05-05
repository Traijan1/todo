# OAuth2 Implementierungsplan für MCP Server (Loco)

Da der MCP (Model Context Protocol) Server derzeit keine robuste Authentifizierung besitzt, ist die Implementierung von OAuth2 (Authorization Code Flow mit PKCE) der sicherste Weg, um den Zugriff zu schützen.

## 1. Architektur-Übersicht

Der Plan sieht vor, den MCP-Server als **Resource Server** zu behandeln, der JWTs (JSON Web Tokens) validiert, die von deinem Loco-Backend (dem **Authorization Server**) ausgestellt wurden.

### Komponenten:
- **Authorization Server:** Dein Loco Backend (`todo` App).
- **Resource Server:** Die MCP Endpunkte innerhalb der Loco App.
- **Client:** Der MCP Client (z.B. Claude Desktop, IDE-Erweiterungen), der Zugriff auf die Tools benötigt.

---

## 2. Phasen der Implementierung

### Phase 1: Datenbank-Erweiterung (Migration)
Wir benötigen Tabellen, um OAuth2-Clients und Authorization-Codes zu verwalten.
- **`oauth_clients`**: `client_id`, `client_secret_hash`, `redirect_uris`, `grant_types`.
- **`oauth_tokens`**: `access_token`, `refresh_token`, `expires_at`, `user_id`.

### Phase 2: Loco Backend Erweiterung
1.  **OAuth2 Controller**: Neue Routen in `src/controllers/auth.rs` oder einem neuen `oauth.rs`:
    - `GET /oauth/authorize`: Validiert Client & User, zeigt Login-Seite (Consent).
    - `POST /oauth/token`: Tauscht Authorization Code gegen Access Token (und Refresh Token).
2.  **PKCE Support**: Implementierung von `code_challenge` und `code_verifier` Validierung (Pflicht für Clients ohne sicheres Secret).

### Phase 3: MCP Endpunkte absichern
Der `McpController` muss so angepasst werden, dass er den `Authorization: Bearer <token>` Header prüft.
- **Middleware**: Erstellung einer Loco-Middleware, die das JWT gegen die Datenbank oder den Secret Key validiert.
- **Scopes**: Einführung von Scopes (z.B. `mcp:read`, `mcp:write`), um den Zugriff granular zu steuern.

---

## 3. Detaillierter Flow (Beispiel Claude Desktop)

1.  **Discovery**: Claude erkennt, dass der Server OAuth benötigt.
2.  **Redirect**: Claude öffnet den Browser: `https://todo.traijan.de/oauth/authorize?client_id=claude&response_type=code...`
3.  **Auth**: Du loggst dich in deiner Todo-App ein.
4.  **Consent**: Du bestätigst: "Darf Claude auf deine Todos zugreifen?"
5.  **Callback**: Browser leitet zurück zu Claude mit einem `code`.
6.  **Token Exchange**: Claude tauscht den `code` gegen ein `access_token`.
7.  **Usage**: Claude sendet bei jedem Tool-Aufruf:
    ```json
    { "Authorization": "Bearer <JWT>" }
    ```

---

## 4. Sicherheits-Checkliste

- [ ] **JWT Signing**: Nutze `RS256` (Asymmetrisch) statt `HS256`, damit der MCP-Teil den Key nicht kennen muss (nur Public Key).
- [ ] **Expiration**: Kurze Lebensdauer für Access Tokens (z.B. 1 Stunde), lange für Refresh Tokens.
- [ ] **HTTPS**: Traefik erzwingt bereits TLS, was für OAuth2 zwingend ist.
- [ ] **Audience Validation**: Sicherstellen, dass das Token explizit für den MCP-Server (`aud: mcp_server`) ausgestellt wurde.

---

## 5. Nächste Schritte in Loco

Um dies in Rust/Loco umzusetzen, empfehle ich die Crate `oxide-auth` oder `openidconnect-rs`.

**Soll ich dir ein Grundgerüst für den OAuth-Controller in Rust erstellen?**
