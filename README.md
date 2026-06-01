# Évaluation TP 1

Si vous utilisez linux, vous devez exécuter la commande suivante
```bash
sudo setenforce 0
```

Voici les étapes à réalisé:
```bash
git clone https://github.com/jaurel-fosset/5a-mmorpg-server.git
docker compose up --build
```

Normalement, ceci devra build les différents instances demandé du TP 1.


Pour simuler une communication avec le Gatekeeper, voici les commandes possibles:
* Pour récupérer l'état de connexion entre le Gatekeeper et le Redis
```bash
curl -s \
//     -w '\n' \                    
//     http://127.0.0.1:3000/health 
```

* Pour récupérer une adresse ip d'un Gameserver disponible:
```bash
curl -s \                
//     -w '\n' \
//     -H 'Content-Type: application/json' \
//     -d '{"username":"foo","password":"1234"}' \
//     http://localhost:3000/login
```
Attention, entrer un password différent de "1234" ne vous retournera aucun serveur.
