# 🪟 Guide de Démarrage Windows - UAIP Hub

Guide complet pour démarrer UAIP Hub sur Windows 10/11.

## 📋 Prérequis

### 1. Installer Docker Desktop

1. Télécharger [Docker Desktop pour Windows](https://docs.docker.com/desktop/install/windows-install/)
2. Exécuter l'installateur
3. Redémarrer votre ordinateur
4. Ouvrir Docker Desktop et attendre qu'il démarre complètement
5. Vérifier l'installation:
   ```powershell
   docker --version
   docker-compose --version
   ```

### 2. Installer Rust

1. Télécharger et exécuter [rustup-init.exe](https://rustup.rs/)
2. Suivre les instructions à l'écran
3. Redémarrer le terminal
4. Vérifier l'installation:
   ```powershell
   rustc --version
   cargo --version
   ```

### 3. Installer Git

1. Télécharger [Git pour Windows](https://git-scm.com/download/win)
2. Installer avec les options par défaut
3. Vérifier:
   ```powershell
   git --version
   ```

## 🚀 Démarrage Rapide

### Option 1: PowerShell (Recommandé)

1. **Ouvrir PowerShell**
   - Appuyer sur `Windows + X`
   - Sélectionner "Windows PowerShell" ou "Terminal"

2. **Cloner le projet**
   ```powershell
   cd $HOME\Documents
   git clone https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP.git
   cd UAIP
   ```

3. **Démarrer UAIP Hub**
   ```powershell
   .\quick-start.ps1
   ```

   **Si vous voyez une erreur d'exécution de script:**
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   .\quick-start.ps1
   ```

### Option 2: Command Prompt (CMD)

1. **Ouvrir CMD**
   - Appuyer sur `Windows + R`
   - Taper `cmd` et appuyer sur Entrée

2. **Cloner et démarrer**
   ```cmd
   cd %USERPROFILE%\Documents
   git clone https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP.git
   cd UAIP
   quick-start.bat
   ```

## ⏱️ Temps de Démarrage

- **Première exécution**: 5-10 minutes
  - Téléchargement des images Docker
  - Compilation du code Rust
  - Construction de tous les services

- **Exécutions suivantes**: 30 secondes
  - Docker utilise le cache
  - Pas de recompilation nécessaire

## 🎯 Accès aux Services

Une fois démarré, accéder aux services:

| Service | URL | Identifiants |
|---------|-----|--------------|
| 🏠 API UAIP Hub | http://localhost:8443 | - |
| 📊 Grafana | http://localhost:3000 | admin / admin |
| 📈 Prometheus | http://localhost:9090 | - |
| 🗄️ PostgreSQL | localhost:5432 | uaip / uaip_password_dev |
| 🔴 Redis | localhost:6379 | - |
| 📨 NATS | localhost:4222 | - |

## 🛠️ Commandes Utiles

### Voir les Logs

```powershell
# PowerShell
docker-compose -f docker-compose.dev.yml logs -f uaip-hub

# Voir tous les logs
docker-compose -f docker-compose.dev.yml logs -f
```

### Arrêter les Services

```powershell
docker-compose -f docker-compose.dev.yml down
```

### Redémarrer

```powershell
docker-compose -f docker-compose.dev.yml restart uaip-hub
```

### Nettoyer Complètement

```powershell
# Arrêter et supprimer tout (volumes inclus)
docker-compose -f docker-compose.dev.yml down -v

# Nettoyer les images Docker
docker system prune -a
```

## 🧪 Tests et Développement

### Lancer les Tests

```powershell
cargo test --workspace
```

### Compiler le Projet

```powershell
# Mode debug (rapide)
cargo build --workspace

# Mode release (optimisé)
cargo build --workspace --release
```

### Vérifier le Code

```powershell
# Formater le code
cargo fmt --all

# Linter (Clippy)
cargo clippy --workspace --all-targets

# Audit de sécurité
cargo audit
```

## ❌ Dépannage

### Erreur: "Docker daemon is not running"

**Solution:**
1. Ouvrir Docker Desktop
2. Attendre que l'icône Docker dans la barre des tâches soit stable (vert)
3. Relancer le script

### Erreur: "Port 8443 is already allocated"

**Solution:**
```powershell
# Voir ce qui utilise le port
netstat -ano | findstr :8443

# Arrêter les services
docker-compose -f docker-compose.dev.yml down

# Relancer
.\quick-start.ps1
```

### Erreur: "Cannot connect to PostgreSQL"

**Solution:**
```powershell
# Vérifier que PostgreSQL est démarré
docker ps | findstr postgres

# Redémarrer PostgreSQL
docker-compose -f docker-compose.dev.yml restart postgres
```

### Les Migrations Échouent

**Solution:**
```powershell
# Arrêter tout
docker-compose -f docker-compose.dev.yml down -v

# Relancer (cela recréera la base de données)
.\quick-start.ps1
```

### Build Docker Très Lent

**Astuces:**
1. Fermer les autres applications gourmandes en CPU
2. Allouer plus de ressources à Docker Desktop:
   - Docker Desktop → Settings → Resources
   - Augmenter CPU et RAM
3. Utiliser WSL 2 si disponible:
   - Docker Desktop → Settings → General → "Use WSL 2 based engine"

### Erreur PowerShell Execution Policy

**Solution:**
```powershell
# En tant qu'utilisateur normal
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# OU exécuter directement en contournant la politique
powershell -ExecutionPolicy Bypass -File .\quick-start.ps1
```

## 🔒 Sécurité

### Pare-feu Windows

Si Windows Defender Firewall bloque Docker:
1. Panneau de configuration → Système et sécurité → Pare-feu Windows Defender
2. Paramètres avancés → Règles de trafic entrant
3. Autoriser "Docker Desktop Backend" et "vpnkit-bridge"

### Antivirus

Si votre antivirus bloque Docker ou Rust:
1. Ajouter des exceptions pour:
   - `C:\Program Files\Docker`
   - `%USERPROFILE%\.cargo`
   - `%USERPROFILE%\.rustup`
   - Votre dossier de projet UAIP

## 💡 Conseils de Performance

### 1. Utiliser WSL 2

Pour de meilleures performances, utilisez WSL 2:
```powershell
# Installer WSL 2
wsl --install

# Configurer Docker pour utiliser WSL 2
# Docker Desktop → Settings → General → Use WSL 2 based engine
```

### 2. SSD Recommandé

Docker fonctionne beaucoup mieux sur un SSD qu'un disque dur.

### 3. Ressources Docker

Allouer suffisamment de ressources:
- **CPU**: Minimum 2 cœurs, recommandé 4+
- **RAM**: Minimum 4 GB, recommandé 8+ GB
- **Disk**: Minimum 20 GB d'espace libre

## 📞 Support

- 🐛 **Bugs**: [GitHub Issues](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues)
- 💬 **Questions**: [GitHub Discussions](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/discussions)
- 📧 **Contact**: [@Hakille](https://github.com/Hakille)

## 🎓 Prochaines Étapes

Une fois UAIP Hub démarré avec succès:

1. 📖 Lire la [Documentation API](http://localhost:8443/docs)
2. 🔍 Explorer le [Dashboard Grafana](http://localhost:3000)
3. 🧪 Tester les [Exemples d'API](./examples/)
4. 🤖 Créer votre premier Agent IA

---

**Créé avec ❤️ par [Hakille](https://github.com/Hakille)**
