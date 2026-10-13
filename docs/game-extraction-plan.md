# Extraction des véhicules depuis ETS2

La chaîne maintenue est la commande Rust `tpv extract` dans `tp-pack`.
Elle utilise les archives de l’installation ETS2 et ConverterPIX comme
exécutable externe, puis la construction et la validation natives des `.tpv`.
Le catalogue, l’import Carsmaniac et Paintjob Packer sont retirés. Les paquets
s’installent manuellement dans la bibliothèque ; aucun parcours marketplace
ou extraction intégrée à l’application n’est ajouté.

```sh
mise exec -- cargo run -p tp-pack --bin tpv -- extract "/chemin/vers/ETS2" \
  --converter "/chemin/vers/converter_pix" --package-version 2.0.0
```

Ajouter `--vehicle man.tgx_2020` ou `--vehicle daf.2021` limite le lot.
Chaque lot a son propre sous-dossier de `dist/`, avec les paquets validés
et `report.json` ; les intermédiaires et journaux restent sous `target/`.
La validation technique et la vérification en jeu sont distinctes. Un lot
incomplet continue les autres véhicules et retourne un code d’échec.

## Références historiques MAN/DAF

Les premiers essais utilisaient ETS2 **1.61.1.1**, les archives de base et
le seul DLC du véhicule, sans les autres DLC de tuning ou de peintures.

| Véhicule | Chemin | Cabines | Accessoires indépendants | Modèles convertis |
|---|---|---|---:|---:|
| MAN TGX récent | `man.tgx_2020` | GX, GM, GN | 27 | 195 |
| DAF NGD | `daf.2021` | XG+, XG, XF | 12 | 172 |

La génération ne lisait ni Paintjob Packer ni les images Carsmaniac.
Ces dernières servaient uniquement à une comparaison visuelle, avec un
pack 1.59 différent du jeu 1.61. Les contrôles techniques avaient réussi :
validation des paquets, sauvegarde/relecture des projets (30 surfaces MAN,
15 DAF), export des mods, dimensions et pixels DDS, identifiants
d’accessoires et références TOBJ. Les rapports initiaux indiquaient une
validation en jeu en attente. L’utilisateur a ensuite confirmé les
configurations vérifiées pour MAN puis DAF ; aucune matrice exhaustive de
toutes les configurations n’a été enregistrée.

Les métadonnées de régression sans ressources du jeu sont conservées dans
`crates/tp-pack/tests/fixtures/game-extraction/validated-baseline.json`,
avec l’inventaire des hypothèses dans le README voisin. Les anciens SVG,
paquets, projets, mods et rapports restent localement sous
`target/prototypes/{man-tgx-game,daf-ngd-game}/run-1/`. Les premières
confirmations ne valident pas automatiquement les nouvelles contributions
DLC ou les différences dues aux variantes UK.

## Règles issues des essais

- Sélection du look et de la variante demandés, puis des pièces visibles
  avec matériau de peinture ordinaire.
- Choix du flux par l’alias `_TEXCOORD1`, sans retournement vertical
  supplémentaire des coordonnées d’image de ConverterPIX.
- Refus des UV alternatifs et shaders non pris en charge.
- Gabarits principaux superposant les options peintes compatibles.
- Cabines en 4096 × 4096 ; dimensions des accessoires déduites des noms
  de découpe des masques, avec refus des tailles absentes ou ambiguës.
- Les listes d’accessoires réglant uniquement les paillettes n’affectent
  pas de texture indépendante : une affectation exige `paint_job_mask`.

Les identités extraites sont `official.ets2.<véhicule>`, dont
`official.ets2.daf.2021`. Les anciens paquets Carsmaniac sont retirés
manuellement, sans migration des placements ni suppression automatique
de bibliothèques ou de projets.

## Cas rencontré pendant la généralisation

Le DLC de peintures hongroises installé (`dlc_flags_hu.scs`) référence,
pour le MAN récent, des accessoires tels que `sideskirt.6x24` dans
`paint_job/accessory/hu_sml_m20_a.sii`, sans définition correspondante
dans les archives installées. La règle stricte refuse actuellement ce
véhicule. Le traitement des références orphelines reste à décider ; aucune
exception silencieuse n’est appliquée.
