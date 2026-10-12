# DAF NGD : deuxième essai d'extraction directe

Source : installation ETS2 **1.61.1.1**, véhicule **`daf.2021`**.
Le profil DAF réutilise l'extracteur MAN, avec le DLC `dlc_daf_2021.scs`
et les modèles `daf_2021`. Les matériaux peints examinés utilisent
`eut2.truckpaint.tsnmapuv` et l'alias UV `_TEXCOORD1`.

172 modèles extérieurs convertis, 53 définitions avec des surfaces peintes,
3 cabines et 12 accessoires indépendants. Aucun fichier Carsmaniac ni
Paintjob Packer n'est lu par la génération. Le pack Carsmaniac 1.59 a
seulement servi à la comparaison visuelle après extraction.

## Fichiers

- [Véhicule `.tpv`](../../target/prototypes/daf-ngd-game/run-1/experimental.game.daf.2021-0.1.0.tpv).
- [Projet de contrôle `.truckpaint`](../../target/prototypes/daf-ngd-game/run-1/daf_ngd_game_uv_probe.truckpaint).
- [Mod de contrôle `.scs`](../../target/prototypes/daf-ngd-game/run-1/daf_ngd_game_uv_probe.scs).
- [Aperçu des 15 gabarits](../../target/prototypes/daf-ngd-game/run-1/templates-overview.png).
- [Comparaison XG+](../../target/prototypes/daf-ngd-game/run-1/xg-plus-comparison.png).
- [Sources et contributeurs de chaque gabarit](../../target/prototypes/daf-ngd-game/run-1/report.json).
- [Vérifications des fichiers](../../target/prototypes/daf-ngd-game/run-1/validation.txt).

## Essai en jeu

Copier le `.scs` dans le dossier `mod` utilisateur d'ETS2, l'activer et
choisir **DAF NGD GAME UV PROBE** sur le DAF nouvelle génération.
Commencer par **XG+, châssis 4×2**, puis vérifier XG et XF. Éviter les mods
qui remplacent ce véhicule pendant le contrôle.

Chaque texture porte le numéro du gabarit, les coordonnées de cellule et
des repères `^ TOP >` :

| Numéro | Surface |
|---|---|
| 00 | XG+ |
| 01 | XG |
| 02 | XF |
| 03 | Rétroviseurs caméra `mirror.cam_semi` |
| 04 | Zone de peinture du modèle `mirror.plast` |
| 05 | Rétroviseurs `mirror.semi` |
| 06–11 | Jupes 4×2 peintes / silver / silver 2, dont variantes XF |
| 12 | Pare-soleil peint XF |
| 13 | Pare-soleil peint XG |
| 14 | Pare-soleil peint XG+ |

Contrôler la cabine à gauche, à droite et derrière, puis les rétroviseurs
standard/caméra, les jupes et le pare-soleil peints. Les autres pièces
peintes compatibles sont incluses dans le gabarit principal. Le modèle
`mirror.plast` a une petite surface avec matériau de peinture ; cela ne
signifie pas que sa coque plastique entière sera peinte.

## Reproduire

Prérequis et binaire ConverterPIX identiques à l'essai MAN. Depuis la
racine du dépôt, utiliser un dossier de sortie neuf :

```sh
python3 experiments/man-tgx-game/extract.py \
  '/mnt/games/SteamLibrary/steamapps/common/Euro Truck Simulator 2' \
  /tmp/man-tgx-converter_pix target/prototypes/daf-ngd-game/run-2 daf.2021
mise exec -- cargo run -p tp-app --release --locked \
  --example man_tgx_game_probe -- target/prototypes/daf-ngd-game/run-2
```

Le nom de l'exemple Rust reste celui du premier essai MAN ; les noms du
projet, du mod et de la peinture sont désormais tirés du package fourni.
Sans quatrième argument, l'extracteur continue de produire le MAN TGX.

Validations réussies : trois tests de l'extracteur, lecture native du `.tpv`,
sauvegarde/relecture des 15 surfaces du projet, export natif du mod,
intégrité ZIP, dimensions et pixels de calibration des 15 DDS, toutes les
références TOBJ, et couverture des 12 identifiants dans chacune des trois
définitions d'accessoires. Le rendu en jeu reste à vérifier. Les limites
restent celles du premier essai : modèles extérieurs, superposition des
options compatibles et exclusion des autres DLC/mods.
