# Guide de contribution — Nouveau parser

## Ajouter une nouvelle source

1. Copier le template :
   ```bash
   cp -r _schema/template.toml catalog/<category>/<source>/vector.toml
   ```

2. Editer `vector.toml` :
   - Adapter la source (syslog, file, http...)
   - Ecrire le VRL de parsing dans le transform `parse_and_normalize`
   - Mapper vers les champs OCSF obligatoires
   - Garder le log source dans `.raw` pour les evenements OCSF valides
   - Router les echecs de parsing vers `raw-logs`, pas vers un index OCSF

3. Creer les tests dans `tests/` (minimum 3) :
   - `nominal.toml` — event standard, tous les champs presents
   - `optional_missing.toml` — champs optionnels manquants
   - `malformed.toml` — input invalide, doit sortir en `raw-logs` avec `parse_status = "failed"` sans sortir en OCSF

4. Creer un `README.md` documentant :
   - Description de la source
   - Format de log attendu
   - Configuration cote source (comment envoyer les logs)
   - Variables d'environnement specifiques
   - Liens vers la doc officielle

5. Valider :
   ```bash
   # Vector 0.54 --no-environment ne desactive PAS l'expansion ${VAR} :
   # passer par ci/validate.sh (injecte des dummy vars) ou exporter soi-meme.
   bash ci/validate.sh
   bash ci/test.sh
   ```

## Champs OCSF obligatoires

Chaque event normalise doit contenir :

| Champ           | Type   | Description                    |
|-----------------|--------|--------------------------------|
| `class_uid`     | int    | Classe OCSF (ex: 4001)        |
| `category_uid`  | int    | Categorie OCSF (ex: 4)        |
| `severity_id`   | int    | 0=Unknown, 1=Info, 2=Low...   |
| `time`          | int    | Epoch milliseconds             |
| `metadata`      | object | `product.name`, `vendor_name`  |
| `tenant_id`     | string | Injecte via `$TENANT_ID`       |
| `datasource_id` | string | Injecte via `$DATASOURCE_ID`   |
| `raw`           | string | Message original conserve      |
| `uid`           | string | UUID partage avec raw-logs si besoin d'investigation |

## Conventions

- Fichiers en TOML
- VRL inline dans le transform (pas de fichier `.vrl` separe)
- Variables runtime en `${ENV_VAR}` avec defaults si applicable
- Logs de test : bruts reels, pas inventes
- Aucun evenement non parse ne doit etre envoye dans un index OCSF
- Les champs OCSF dynamiques doivent etre routes vers l'index Quickwit qui correspond a leur `class_uid`

## Convention collecteur / parser

Kolektor separe la recuperation des logs et leur normalisation :

- un **collecteur** recupere les logs depuis la source : syslog, fichier, API pull
  avec curseur, object storage + queue, Event Hub/EventBridge, ou Logpush HTTP ;
- un **parser** Vector transforme un format brut canonique en OCSF et route vers
  les index Quickwit.

Pour les sources cloud/SaaS, ne pas enfouir la pagination, OAuth, retry/backoff
ou gestion de curseur dans le VRL. Preferer un collecteur dedie qui depose du
JSON line-delimited ou pousse des objets JSON vers Vector. Le parser doit rester
testable avec des fixtures brutes et reutilisable quel que soit le transport.

## Convention raw / OCSF / raw-logs

Chaque evenement OCSF valide garde une copie du log source dans `.raw`.
Pour les sources texte (`file`, payload syslog), `.raw` doit etre le message original
ou la ligne reconstruite la plus proche possible. Pour les sources JSON deja decodees
par Vector (`http_server encoding = "json"`), capturer `raw_msg = encode_json(.)`
avant d'ajouter `tenant_id`, `datasource_id` ou les champs OCSF.

Chaque evenement valide porte aussi un `uid` :

```vrl
_ts  = to_string(.timestamp) ?? ""
_pid = if .procid != null { "[" + to_string!(.procid) + "]" } else { "" }
.raw = _ts + " " + (string(.hostname) ?? "") + " " + (string(.appname) ?? "") + _pid + ": " + _msg
.uid = uuid_v4()
```

Les evenements parses ne sont pas recopies systematiquement dans `raw-logs` :
leur brut est deja dans `.raw`. `raw-logs` sert a isoler les echecs de parsing
et les formats non supportes.

Pattern attendu pour les echecs :

```toml
[transforms.filter_failed]
type      = "filter"
inputs    = ["parse_and_normalize"]
condition = '.class_uid == 0'

[transforms.raw_failed]
type   = "remap"
inputs = ["filter_failed"]
source = '''
  . = {
    "uid":           .uid,
    "time":          .time,
    "received_time": .received_time,
    "tenant_id":     .tenant_id,
    "datasource_id": .datasource_id,
    "source_type":   "category/source",
    "parser":        "source",
    "parse_status":  "failed",
    "parse_error":   "reason_code",
    "raw":           .raw
  }
'''
```

Le sink `raw_failed` envoie vers `${QUICKWIT_ENDPOINT}/api/v1/raw-logs/ingest`.

Le `uid` permet a Kontrol de correler un event OCSF normalise avec sa ligne brute
originale quand elle existe dans un autre flux, et donne un identifiant stable
pour les evenements de quarantaine.

## Routage dynamique quand `class_uid` varie

Si une source produit plusieurs classes OCSF (ex: auditd = 1003 + 3002,
windows-evtx = 3001/3002/1003), il faut un transform `route` + un sink par
index Quickwit cible. Un sink unique vers `ocsf-endpoint` avec des events 3002
dedans = donnees au mauvais endroit. Voir [catalog/linux/auditd/vector.toml](../catalog/linux/auditd/vector.toml).

Le `manifest.yaml` doit declarer les sorties multiples avec `ocsf_outputs` :

```yaml
display_name: Windows Sysmon
default_port: 8515
ocsf_outputs:
  - class_uid: 1003
    category_uid: 1
    index: ocsf-endpoint
    route: endpoint
  - class_uid: 4001
    category_uid: 4
    index: ocsf-network
    route: network
  - class_uid: 4003
    category_uid: 4
    index: ocsf-dns
    route: dns
```

Pour une source mono-classe, les champs historiques `ocsf_class_uid` et
`ocsf_category_uid` restent acceptes ; le seed genere automatiquement une sortie.

## Tests attendus

Les tests doivent verifier :

- un cas nominal jusqu'au transform normalise ;
- un cas optionnel avec champs absents ;
- un cas malformed avec `.class_uid == 0` au transform de parsing ;
- une sortie du transform `raw_failed` avec `parse_status = "failed"`,
  `source_type`, `parse_error`, `raw`, `uid`, `tenant_id` et `datasource_id` ;
- pour les parsers multi-classes, au moins un cas par route (`endpoint`,
  `identity`, `network`, `dns`, etc.).
