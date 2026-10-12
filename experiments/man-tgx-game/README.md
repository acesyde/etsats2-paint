# MAN TGX : extraction directe du jeu, prototype

Question : peut-on fabriquer un `.tpv` utilisable sans les données ni les
templates Carsmaniac ? Le premier essai produit les gabarits et un mod de
calibration ; la validation dans ETS2 reste à faire.

Le même extracteur prend aussi `daf.2021` en quatrième argument. Voir
[l'essai DAF NGD](DAF-NGD.md) pour les fichiers et la procédure de contrôle.

## Résultat local

Source : installation ETS2 **1.61.1.1**, véhicule **`man.tgx_2020`**.
195 modèles extérieurs convertis, 42 définitions avec des surfaces peintes,
3 gabarits de cabine et 27 gabarits d'accessoires indépendants.

Les fichiers sont dans
[`target/prototypes/man-tgx-game/run-1/`](../../target/prototypes/man-tgx-game/run-1/).

- [`experimental.game.man.tgx_2020-0.1.0.tpv`](../../target/prototypes/man-tgx-game/run-1/experimental.game.man.tgx_2020-0.1.0.tpv) : véhicule à installer dans TruckPaint.
- [`man_tgx_game_uv_probe.truckpaint`](../../target/prototypes/man-tgx-game/run-1/man_tgx_game_uv_probe.truckpaint) : projet prêt à ouvrir, avec les 30 surfaces et des grilles de contrôle.
- [`man_tgx_game_uv_probe.scs`](../../target/prototypes/man-tgx-game/run-1/man_tgx_game_uv_probe.scs) : mod de contrôle exporté avec le code de TruckPaint.
- [`templates-overview.png`](../../target/prototypes/man-tgx-game/run-1/templates-overview.png) : correspondance entre les numéros et les pièces.
- [`gx-comparison.png`](../../target/prototypes/man-tgx-game/run-1/gx-comparison.png) : comparaison visuelle, Carsmaniac 1.59 à gauche, extraction 1.61 à droite. Le pack n'a servi qu'à cette comparaison.
- [`report.json`](../../target/prototypes/man-tgx-game/run-1/report.json) : sources et modèles contribuant à chaque gabarit.
- [`validation.txt`](../../target/prototypes/man-tgx-game/run-1/validation.txt) : vérifications effectuées.

## Essai dans ETS2

1. Copier le `.scs` dans le dossier `mod` utilisateur d'ETS2 et l'activer
   dans le gestionnaire de mods. Éviter les autres mods qui remplacent le MAN TGX
   pendant cet essai.
2. Personnaliser le MAN TGX récent, cabine GX, et choisir la peinture
   **MAN TGX GAME UV PROBE**. Vérifier ensuite GM et GN.
3. Chaque surface porte un numéro, des coordonnées de cellule et des repères
   `^ TOP >`. Les surfaces des cabines sont **00 GX**, **01 GM**, **02 GN**.
   L'aperçu des templates indique les numéros **03–29** des accessoires.
4. Vérifier notamment les pare-chocs standard/Duty, les déflecteurs, les
   rétroviseurs standard/caméra (dont GN), les jupes, les garde-boue selon le
   châssis et les feux arrière peints. Noter les pièces restées unies, les
   numéros inattendus et les repères inversés. Une capture avec la cabine et
   l'accessoire choisis permet de retrouver leur définition dans le rapport.

## Reproduire

Prérequis : Python avec Pillow et DejaVu Sans Bold ;
[ConverterPIX](https://github.com/mwl4/ConverterPIX) pour Linux.
Le binaire de cet essai est `/tmp/man-tgx-converter_pix`, SHA-256
`fba5a52f763d07d3dc110dea4980fe2c7e656a8c0afa05b6b7a4ab4373dc950d`.

Depuis la racine du dépôt, choisir un nouveau dossier de sortie :

```sh
python3 experiments/man-tgx-game/extract.py \
  '/mnt/games/SteamLibrary/steamapps/common/Euro Truck Simulator 2' \
  /tmp/man-tgx-converter_pix target/prototypes/man-tgx-game/run-2
mise exec -- cargo run -p tp-app --release --locked \
  --example man_tgx_game_probe -- target/prototypes/man-tgx-game/run-2
python3 -m unittest discover -s experiments/man-tgx-game -p 'test_*.py'
```

L'extraction reprend un dossier incomplet, mais refuse de remplacer un
véhicule déjà généré. Les fichiers du jeu ne sont jamais modifiés.
Les fichiers dérivés du jeu restent sous `target/`, hors des fichiers Git.

## Ce qui est vérifié et ce qui reste à apprendre

- Les identifiants viennent des unités SII et des `acc_list` du jeu.
- Les modèles, looks et variantes viennent des définitions du véhicule.
- Les faces visibles avec un matériau `eut2.truckpaint` utilisent le flux
  portant l'alias `_TEXCOORD1`. Les UV de ConverterPIX sont déjà en
  coordonnées d'image : aucun retournement vertical supplémentaire.
- Les SVG conservent les coordonnées UV. Les cabines sont proposées en
  4096 × 4096 ; c'est un choix de résolution de sortie. Les dimensions des
  accessoires viennent des noms de textures découpées des peintures SCS.
- Le lecteur de packages valide le `.tpv`. Le projet est sauvegardé puis
  relu. L'exporteur existant écrit le mod. Les références TOBJ, les 30 DDS,
  leurs dimensions, leurs pixels de calibration et les 27 identifiants
  d'accessoires sont vérifiés après export.

Limites : shaders MAN TGX ordinaires, modèles extérieurs, archives de base
et DLC MAN seulement. Les UV alternatifs sont refusés. Les gabarits de
cabine superposent toutes les options peintes compatibles : certains traits
se chevauchent. Les noms sont techniques et les accessoires ne sont pas
regroupés. Ne pas généraliser cette règle d'UV à d'autres véhicules avant
un essai en jeu ; des modèles intérieurs et d'autres DLC/mods restent à
couvrir si le contrôle révèle des pièces manquantes.
