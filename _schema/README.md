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

3. Creer les tests dans `tests/` (minimum 3) :
   - `nominal.toml` — event standard, tous les champs presents
   - `optional_missing.toml` — champs optionnels manquants
   - `malformed.toml` — input invalide, doit etre droppe sans crash

4. Creer un `README.md` documentant :
   - Description de la source
   - Format de log attendu
   - Configuration cote source (comment envoyer les logs)
   - Variables d'environnement specifiques
   - Liens vers la doc officielle

5. Valider :
   ```bash
   vector validate --no-environment catalog/<category>/<source>/vector.toml
   vector test catalog/<category>/<source>/vector.toml catalog/<category>/<source>/tests/*.toml
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

## Conventions

- Fichiers en TOML
- VRL inline dans le transform (pas de fichier `.vrl` separe)
- Variables runtime en `${ENV_VAR}` avec defaults si applicable
- Logs de test : bruts reels, pas inventes
