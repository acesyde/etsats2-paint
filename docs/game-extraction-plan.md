# Extraction des véhicules depuis ETS2

La chaîne maintenue est la commande Rust `tpv extract` dans `tp-pack`.
Elle utilise les archives de l’installation ETS2 et ConverterPIX comme
exécutable externe, puis la construction et la validation natives des `.tpv`.
Le catalogue, l’import Carsmaniac et Paintjob Packer sont retirés. Les paquets
s’installent manuellement dans la bibliothèque ; aucun parcours marketplace
ou extraction intégrée à l’application n’est ajouté.

```sh
mise exec -- cargo run --release -p tp-pack --bin tpv -- extract "/chemin/vers/ETS2" \
  --converter "/chemin/vers/converter_pix" --package-version 2.0.0
```

Ajouter `--vehicle man.tgx_2020` ou `--vehicle daf.2021` limite le lot.
Chaque lot a son propre sous-dossier de `dist/`, avec les paquets validés
et `report.json` ; les intermédiaires et journaux restent sous `target/`.
La validation technique et la vérification en jeu sont distinctes. Un lot
incomplet continue les autres véhicules et retourne un code d’échec.
Les références manquantes sont détaillées dans `missing_dependencies` du
rapport, avec les définitions et archives qui les mentionnent lorsque connues.
`expected_dlc: null` et `dlc_origin: "unknown"` signalent une origine DLC
indéterminée : une référence orpheline ne prouve pas qu’un DLC manque.

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
- Le prototype refusait les UV alternatifs ; l’outil maintenu prend désormais
  en charge les familles décrites ci-dessous et refuse les cas non reconnus.
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
dans les archives installées. Ces références sans
définition installée sont désormais exclues des affectations, avec
`disposition: "ignored_reference"` dans `missing_dependencies`. Elles ne
bloquent pas le paquet. Une définition installée sans géométrie peinte
exploitable ou un fichier requis absent reste bloquant. Le diagnostic ne
prouve pas qu’un DLC manque.

## Régression avec l’outil Rust

Après exclusion diagnostiquée des références sans définition installée,
le MAN est généré avec ses 3 cabines et 27 accessoires. Les 30 SVG sont
identiques octet pour octet à ceux du prototype ; identifiants et dimensions
correspondent aussi. Les 19 références absentes du DLC de peintures hongroises
restent dans le rapport, sans être attribuées à un DLC manquant.
Le DAF conserve ses 3 cabines et 12 accessoires, avec ses 15 SVG également
identiques au prototype. Les tests de l’application vérifient explicitement
l’installation manuelle des deux paquets, la création avec toutes les cabines,
la sauvegarde/relecture du projet et l’export du mod, y compris dimensions DDS
et contenu des définitions et TOBJ. Ces contrôles ne sont pas une nouvelle
validation en jeu. Les résultats locaux figurent dans `report.json`.

## Familles de shaders et choix des UV

Les règles sont communes aux véhicules : `truckpaint`, `truckpaint.tsnmap` et
`truckpaint.tsnmapuv`, avec les options `airbrush`, `flipflake` et `altuv`, et les suffixes
de test alpha `.a` ou `.asafew.a`. Ces suffixes modifient l’opacité, pas les
liaisons UV. Le gabarit trace le maillage avant le masque d’opacité par pixel. Les matériaux
peints visibles des cabines déterminent un mode commun : `_TEXCOORD1` pour
le normal, `_TEXCOORD2` pour l’alternatif. Une contradiction entre cabines
ou dans leurs matériaux peints entraîne un refus. Tous les accessoires
utilisent le mode retenu, même si leur matériau source indique un autre mode.
Le manifeste porte le même choix dans `game.alt_uv`, déjà utilisé par l’export
pour `alternate_uvset`. Les flux UV restent choisis par leur alias, jamais
par leur numéro de flux. Les modèles `detail_model` et leur réutilisation UK
sont aussi lus depuis les définitions.

Sources consultées :

- [SCS Blender Tools, shader.py, commit 34dff5239e0f46512af0400a84f6effa5933d4ab](https://github.com/SCSSoftware/BlenderTools/blob/34dff5239e0f46512af0400a84f6effa5933d4ab/addon/io_scs_tools/internals/shaders/shader.py) : sélection du texcoord 1 ou 2 pour la peinture et 0 pour la texture de base.
- [Presets et options des shaders SCS, même commit](https://github.com/SCSSoftware/BlenderTools/blob/34dff5239e0f46512af0400a84f6effa5933d4ab/addon/io_scs_tools/shader_presets.txt) : les options de normal map modifient leurs propres coordonnées ; airbrush exporte les coordonnées de peinture 1 et 2.
- [Définition accessory_paint_job_data](https://modding.scssoft.com/wiki/Documentation/Engine/Units/accessory_paint_job_data) : `alternate_uvset` choisit les UV alternatifs et la configuration de peinture détermine le matériau résultant.
- [ConverterPIX, commit 3cd4e73a86d0c6bd28e117664c50a36cabeccf38](https://github.com/mwl4/ConverterPIX/blob/3cd4e73a86d0c6bd28e117664c50a36cabeccf38/src/model/model.cpp) : les alias `_TEXCOORD…` proviennent du masque des coordonnées du modèle.

Le test local ignoré `replay_validated_game_templates` relit les intermédiaires
indiqués par `TRUCKPAINT_BASELINE_REPORTS` (liste de chemins de rapports,
séparés par le séparateur de la plateforme), régénère leurs SVG et exige une
égalité octet pour octet avec les TPV validés. Il ne requiert ni nouvelle
conversion ni ressources du jeu dans le dépôt.

La variante `truckpaint.tsnmapuv.asafew.a` est présente sur la prise d’air
`fh16_aero` du Volvo FH 2024. Le [code SCS de remappage alpha](https://github.com/SCSSoftware/BlenderTools/blob/34dff5239e0f46512af0400a84f6effa5933d4ab/addon/io_scs_tools/internals/shaders/flavors/asafew.py)
et la [description des options alpha](https://modding.scssoft.com/wiki/Documentation/Tools/SCS_Blender_Tools/Supported_shaders#asafew)
confirment un traitement d’opacité/spécularité distinct du choix des UV.

Après ajout des familles communes, le Mercedes Actros 2014 est généré
avec 4 textures de cabines (BigSpace, GigaSpace, GigaSpace 8×4 et StreamSpace)
et 24 accessoires indépendants. `tpv check` passe ; les 164 contributions
de surfaces peintes rapportées utilisent `_TEXCOORD2`, en accord avec
`game.alt_uv: true`. Aucun contrôle dans l’application ou en jeu n’a été
effectué pour ce nouvel essai limité à la production du TPV.

Le Volvo FH 2024 est aussi généré avec les 12 définitions de cabines
installées et 51 accessoires indépendants. Les 813 contributions de surfaces
peintes rapportées utilisent `_TEXCOORD1`, en accord avec `game.alt_uv: false`.
La validation native passe ; une référence sans définition est exclue et
consignée dans le rapport. Ce second essai reste limité au TPV, sans
installation dans l’application, export de mod ni vérification en jeu.
Les deux nouveaux paquets exigent donc un contrôle visuel en jeu.

Les résultats techniques des essais se trouvent dans les lots locaux ignorés :
`dist/ets2-1792080369348096785-22279-0/` (Actros) et
`dist/ets2-1792080697716765846-2-0/` (Volvo), avec leurs diagnostics et
contrôles locaux dans `report.json`. La CI complète et le build de la
documentation passent après ces extensions. Ces deux essais individuels sont complétés par la passe intégrale décrite
ci-dessous.

## Passe intégrale ETS2

La commande maintenue a traité les 26 camions découverts dans ETS2 1.61.1.1,
avec tous les DLC installés et la version de paquet explicite 2.0.0. Le lot
`dist/ets2-1792081863314453327-2-0/` contient 11 TPV et son `report.json`.
Les 15 refus produisent un code de sortie 1 : le lot reste incomplet.
Aucun paquet partiel ou fichier temporaire n’est présent dans les sorties.

| Véhicule | Cabines | Accessoires indépendants |
|---|---:|---:|
| DAF NGD (`daf.2021`) | 3 | 12 |
| DAF XD | 3 | 6 |
| DAF XF Electric | 1 | 0 |
| Iveco S-Way | 1 | 2 |
| MAN TGX 2020 | 3 | 27 |
| MAN TGX Euro 6 | 3 | 25 |
| Mercedes Actros 2014 | 4 | 24 |
| Renault E-Tech T | 1 | 0 |
| Volvo FH16 2012 | 4 | 42 |
| Volvo FH 2021 | 4 | 45 |
| Volvo FH 2024 | 12 | 51 |

Les 11 fichiers passent individuellement `tpv check`. Les quatre paquets déjà
produits séparément (MAN TGX 2020, DAF NGD, Actros 2014 et Volvo FH 2024)
sont reproduits octet pour octet. Le rapport conserve ces contrôles locaux,
les empreintes des paquets, les sources, les références exclues et les
coordonnées UV sélectionnées. Les nouveaux véhicules restent à vérifier
en jeu ; cette passe ne les installe pas dans l’application et n’exporte
pas de mods.

Les refus sont des limites rencontrées par l’extracteur, sans conclure que les
données du jeu sont incorrectes :

- Huit variantes `flipflake` non prises en charge : DAF XF, Iveco Hi-Way,
  Iveco Stralis, MAN TGX, Mercedes Actros, Renault Magnum, Renault Premium
  et Scania R.
- Quatre cas de coordonnées UV refusées : DAF XF Euro 6, Renault T,
  Scania Streamline et Volvo FH16.
- Deux dimensions contradictoires pour `r_fender.8x4_p` (1024×512 ou
  2048×512) : Scania R 2016 et S 2016.
- Une dimension absente pour `r_bumper.stock_p` : Scania S 2024 Electric.

Les définitions et modèles précis de chaque refus sont dans `report.json`.
Aucune règle n’a été assouplie pendant cette passe.


## Traitement des refus de la première passe

La documentation [SCS sur truckpaint](https://modding.scssoft.com/wiki/Documentation/Tools/SCS_Blender_Tools/Supported_shaders#truckpaint)
indique que `flipflake` et `airbrush.flipflake` n’exigent pas de données exportées
supplémentaires. Ces options utilisent désormais les mêmes règles de géométrie
et d’alias UV que les autres matériaux truckpaint, sans profil par camion.

Les quatre refus UV correspondent à des coordonnées finies sur des sommets
utilisés, hors de [0, 1] : U jusqu’à −2,12 sur le DAF XF Euro 6, V jusqu’à
−1,09 sur le Renault T, U jusqu’à 4,21 sur le Scania Streamline et V jusqu’à
1,67 sur le Volvo FH16 2009. Un autre canal ne résout pas les trois accessoires.
L’outil conserve les coordonnées et le canal sélectionné ; le cadre du SVG
limite le maillage visible. Il n’applique ni modulo, ni écrêtage des sommets,
ni changement opportuniste de canal. Le rapport ajoute `uv_bounds` et
`out_of_bounds_vertices` à chaque surface ; ces cas exigent des essais en jeu.
Les coordonnées non finies et celles non représentables par la quantification
des arêtes restent refusées.

Les conflits Scania R/S 2016 ne sont pas résolus en séparant les modes UV :
le mode ordinaire contient lui-même 1024×512 et 2048×512 pour
`r_fender.8x4_p`. La seconde taille vient de `mg8x4a_s_r_c.sii` /
`mg8x4a_s_s_c.sii` dans `def.scs`. L’extraction des DDS correspondants confirme les deux tailles pour R et S :
il ne s’agit pas d’une erreur dans les noms. La majorité ne permet pas de
déclarer l’autre taille fausse ; le refus reste nécessaire faute de règle validée.

Le Scania électrique n’indique que
`electric/shared_a4cfe8ff.tobj` pour trois accessoires peints. L’extraction
de ce TOBJ confirme une texture 4×4 et `clamp_to_edge` sur les deux axes ;
elle ne fournit pas une résolution de gabarit exploitable. Aucune taille
n’est inventée. `dimension_diagnostics` détaille désormais tous les accessoires
concernés, tailles candidates, masques et définitions/archives qui les référencent,
au lieu de ne révéler que le premier accessoire bloquant.


Après correction des UV, le Renault T atteint la couverture et révèle un
autre refus : `s_guard.renault_pa`, défini dans
`accessory/s_guard/renault_paint.sii` du DLC de tuning Renault, référence
`/vehicle/truck/renault_t/accessory/s_guard/renault_paint.pmd`. Son matériau
`eut2.dif.spec.weight.add.env.tsnmapuv.paint.a` est une teinte de couleur de
base, pas un matériau truckpaint à masque de livrée. Le
[code SCS du flavor paint](https://github.com/SCSSoftware/BlenderTools/blob/34dff5239e0f46512af0400a84f6effa5933d4ab/addon/io_scs_tools/internals/shaders/flavors/paint.py)
multiplie la couleur diffuse par la couleur de base sans texture de livrée.
Les overrides qui lui affectent un masque ne suffisent pas à établir un
mapping UV de livrée. Le refus des accessoires définis sans couverture
exploitable est conservé et testé avec un faux convertisseur ; aucun
accessoire n’est assimilé arbitrairement à truckpaint ni omis d’un paquet.


## Passe après correction des refus

Le lot optimisé `dist/ets2-1792086158559763178-2-0/` traite les 26 véhicules :
**22 TPV validés et 4 refus**, avec un code de sortie 1 pour le lot incomplet.
Les 11 paquets de la première passe sont identiques octet pour octet. Les
45 SVG MAN/DAF restent identiques aux références validées. Les 22 paquets
passent individuellement `tpv check` ; leurs 388 gabarits ont des arêtes
visibles dans le cadre de leur texture et leurs alias UV concordent avec
`game.alt_uv`. Aucun TPV partiel ou fichier temporaire n’est publié.

Les 11 véhicules supplémentaires sont :

| Véhicule | Cabines | Accessoires indépendants |
|---|---:|---:|
| `daf.xf` | 3 | 7 |
| `daf.xf_euro6` | 3 | 8 |
| `iveco.hiway` | 1 | 0 |
| `iveco.stralis` | 3 | 0 |
| `man.tgx` | 3 | 0 |
| `mercedes.actros` | 3 | 33 |
| `renault.magnum` | 1 | 0 |
| `renault.premium` | 2 | 0 |
| `scania.r` | 4 | 5 |
| `scania.streamline` | 4 | 5 |
| `volvo.fh16` | 3 | 27 |

Les quatre refus restants sont documentés dans `report.json` :

- Renault T : la géométrie UV auparavant refusée passe, puis
  `s_guard.renault_pa` bloque la couverture pour le motif décrit ci-dessus.
- Scania R/S 2016 : chacun présente trois conflits 1024×512 / 2048×512
  (`r_fender.8x4_p`, `r_fendr_top.8x4_p`, `r_fendr_top.low_8x4_p`).
- Scania S 2024 Electric : trois accessoires sans dimensions de découpe
  (`r_bumper.stock_p`, `r_fender.4x2_p`, `sunshld.stock`).

Les cinq véhicules générés avec des UV débordants sont le DAF XF Euro 6,
l’Iveco Hi-Way, le Scania R, le Scania Streamline et le Volvo FH16 2009.
Leurs sélections, bornes et nombres de sommets sont consignés dans le
rapport, avec les vérifications locales et l’analyse des refus. Ces véhicules
exigent une attention particulière en jeu. Tous les nouveaux TPV restent à
contrôler en jeu ; aucun nouveau parcours d’installation/export dans
l’application n’a été effectué pour eux. Les essais historiques MAN/DAF ne
valident pas ces nouvelles configurations.

Le formatage, le lint, les tests workspace et le build release passent via
`mise run ci`, ainsi que le build des 89 pages de documentation en quatre
langues. Les tests GPU/jeu ignorés ne sont pas comptés comme réussis.
