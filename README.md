# Évaluation TP 2

Si vous utilisez linux, vous devez exécuter la commande suivante
```bash
sudo setenforce 0
```

Voici les étapes à réalisé:
```bash
git clone https://github.com/jaurel-fosset/5a-mmorpg-server.git
docker build -t spatial-server -f spatial-server/Dockerfile . 
docker build -t broker -f brocker/Dockerfile . 
docker build --provenance=false -t game_server -f game_server/Dockerfile . 
```

Normalement, ceci devra build les différents images qui seront lancé par l'orchestrator.
Une fois tout ceci fait, vous devez réalisé cette commande pour lancer tous les services:
```bash
cargo run --package orchestrator
```
Attention: si des containers existent, l'orchestrator va planter. Assurez vous de bien supprimer tous les containers avant de lancer le programme.
Normalement, l'orchestrator va lancer le broker-service, le redis-dns-service et le spatial-service.
Le spatial-service va ensuite demander à l'orchestrator un shard à son lancement, et après quelques instant, le service "shard-1001" devrait apparaître.

Ensuite, vous pouvez lancer le client:
```bash
cargo run --package client
```

Une fenêtre devrait s'ouvrir avec une interface simple ip/port. Inserez "127.0.0.1" pour l'ip et "10001" pour le port puis connectez vous.
Vous appercevrez dans les logs du clients les messages "positions" concernant le client connecté. (attention, l'affichage est sous forme de bit)
Pour vous déplacer, vous pouvez utilisez les flèches directionnelles.
Vous pourrez voir la mise à jour dans les logs du client ou bien dans les logs de la shard.

Vous pouvez ensuite instancier d'autres clients pour apercevoir le split de shard.
Le spatial-service va demander le lancement de plusieurs shard, et ils seront lancé par l'orchestrator.
Vous pourrez ensuite voir les différentes shard créé ("shard-XXXX") et vous pourrez tester le switch d'authority.

Note: la shard initiale gère une zone en 100x100 mais il ne gère pas lorsqu'on va en dehors de la limite.

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