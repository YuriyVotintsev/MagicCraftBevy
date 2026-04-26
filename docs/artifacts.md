# Artifact Pool

Стартовый пул артефактов для авто-рогалика. **200 штук**, разбиты по
категориям из GDD (`docs/auto-roguelite-gdd.md`).

Это рабочий список для дизайна: имена и числовые значения — placeholder'ы,
будут балансироваться при реализации. Цель доки — задать **объём пула**
и **разнообразие эффектов**, чтобы пула хватало на много ран без повторов
и чтобы билды получались разные.

## Правила пула (из GDD)

- Только positive-or-neutral эффекты. Никаких cursed/негативов.
- Каждый артефакт самодостаточен: никаких evolution-цепочек (X + Y → Z).
- Healing только через явные артефакты (regen-артефакт, lifesteal, on-kill heal).
- Описание одной строкой, без подробных tooltip'ов.
- Конфликтуют только артефакты, занимающие единственный слот / дискретное
  состояние; всё остальное стэкается.

## Распределение по тирам

Целевой mix (приблизительно):
- **common** ~40% — базовые стат-бусты и простые модификаторы
- **rare** ~30% — заметные эффекты, начальные exotics
- **epic** ~20% — сильные модификаторы, кастомные механики
- **legendary** ~10% — game-changing эффекты, ультимативные пушки

## Распределение по категориям (200 = 40 + 30 + 12 + 25 + 15 + 25 + 10 + 43)

| Категория | Кол-во |
|---|---|
| Стат-модификаторы | 40 |
| Поведение снаряда | 30 |
| Замены основной атаки | 12 |
| On-hit | 25 |
| On-kill | 15 |
| Дефенсивы | 25 |
| Healing | 10 |
| Exotic / unique | 43 |

> Источники вдохновения для механик: классика arena-shooter-роуглайтов,
> в первую очередь *The Binding of Isaac: Rebirth/Repentance* (tear effects,
> familiars, weapon transformations типа Brimstone / Tech X / Mom's Knife
> / Dr. Fetus / Epic Fetus / Trisagion / Death's Touch).

---

## Стат-модификаторы (40)

| # | Name | Tier | Effect |
|---|------|------|--------|
| 1 | `Sharpened Core` | common | +15% damage |
| 2 | `Burning Edge` | common | +20% damage |
| 3 | `Heavy Tip` | common | +5 flat damage to every shot |
| 4 | `Lead Heart` | rare | +12 flat damage to every shot |
| 5 | `Forged Spike` | rare | +30% damage |
| 6 | `Razor Bloom` | rare | +40% damage |
| 7 | `Apex Bloom` | epic | +60% damage |
| 8 | `Crimson Surge` | legendary | +100% damage |
| 9 | `Volcanic Vein` | epic | +25 flat damage to every shot |
| 10 | `Fervor` | rare | +50% damage while above 75% HP |
| 11 | `Last Stand` | rare | +50% damage while below 25% HP |
| 12 | `Adrenaline Loop` | epic | +30% damage for 3s after taking a hit |
| 13 | `Hummingbird Pulse` | common | +20% attack speed |
| 14 | `Frantic Trigger` | rare | +35% attack speed |
| 15 | `Stormhand` | epic | +60% attack speed |
| 16 | `Heart of Tempest` | legendary | +100% attack speed |
| 17 | `Rolling Thunder` | rare | +5% attack speed per second of continuous fire (max +50%) |
| 18 | `Quickdraw Sigil` | common | +25% attack speed for the first second of firing |
| 19 | `Lucky Glint` | common | +10% crit chance |
| 20 | `Hawk's Mark` | rare | +20% crit chance |
| 21 | `Falcon's Gaze` | epic | +35% crit chance |
| 22 | `Coup de Grace` | legendary | Crit chance also adds to crit damage 1:1 |
| 23 | `Doubled Edge` | rare | Crit damage +50% (200% → 250%) |
| 24 | `Executioner` | epic | Crits double their bonus damage when target is below 30% HP |
| 25 | `Razor Memory` | rare | Each non-crit increases your next crit chance by 5% (resets on crit) |
| 26 | `Iron Liver` | common | +25 max HP |
| 27 | `Stone Heart` | rare | +50 max HP |
| 28 | `Worldcore` | epic | +100 max HP |
| 29 | `Titan's Pulse` | legendary | +50% max HP |
| 30 | `Wild Vitality` | common | +15% max HP |
| 31 | `Quicksilver Boots` | common | +15% movement speed |
| 32 | `Featherstep` | rare | +25% movement speed |
| 33 | `Slipstream` | epic | +40% movement speed |
| 34 | `Wind Cloak` | rare | +30% movement speed for 2s after each kill |
| 35 | `Storm Runner` | epic | +2% movement speed per second of continuous movement (max +40%) |
| 36 | `Anointed Sigil` | common | -10% all cooldowns |
| 37 | `Time Shard` | rare | -20% all cooldowns |
| 38 | `Chronoweave` | epic | -35% all cooldowns |
| 39 | `Wind Tear` | common | +25% projectile speed |
| 40 | `Lightspike` | rare | +50% projectile speed |

## Поведение снаряда (30)

Стэкаемые модификаторы базового снаряда. Все совместимы между собой.

| # | Name | Tier | Effect |
|---|------|------|--------|
| 41 | `Twin Burst` | common | +1 projectile per shot (parallel) |
| 42 | `Triple Cannon` | rare | +2 projectiles per shot (spread) |
| 43 | `Five Fang` | epic | +4 projectiles per shot (fan) |
| 44 | `Spray and Pray` | legendary | +8 projectiles in wide spread (-20% damage per projectile) |
| 45 | `Backfire` | rare | Each shot also fires one projectile backward |
| 46 | `Crossfire` | epic | Each shot also fires two projectiles to your left and right |
| 47 | `Stardust Volley` | legendary | Each shot also fires 6 projectiles in a full circle around you |
| 48 | `Pierce Tip` | common | Projectiles pierce 1 enemy |
| 49 | `Ironpoint` | rare | Projectiles pierce 3 enemies |
| 50 | `Lance of Ages` | epic | Projectiles pierce all enemies in their path |
| 51 | `Ricochet Cap` | common | Projectiles bounce off walls 1 time |
| 52 | `Pinball Core` | rare | Projectiles bounce off walls 3 times |
| 53 | `Eternity Edge` | epic | Projectiles bounce off walls until they expire |
| 54 | `Skipstone` | rare | Projectiles bounce between enemies (up to 3) |
| 55 | `Carom Lord` | epic | Projectiles bounce between enemies (up to 6) |
| 56 | `Lock-On Lens` | common | Projectiles weakly home in on the nearest enemy |
| 57 | `Predator's Eye` | rare | Projectiles strongly home (sharp turns) |
| 58 | `Wraith Tracker` | epic | Projectiles pass through walls and home indefinitely |
| 59 | `Splinter Round` | rare | Projectiles split into 3 fragments on impact (50% damage each) |
| 60 | `Fissure Round` | epic | Projectiles split into 6 fragments on impact |
| 61 | `Mortar Round` | rare | Projectiles deal small AOE on impact |
| 62 | `Volcano Round` | epic | Projectiles explode in a large AOE on impact |
| 63 | `Boomerang Core` | rare | Projectiles return to you after reaching max range |
| 64 | `Spiral Path` | epic | Projectiles travel in a spiral pattern |
| 65 | `Wide Bullet` | common | Projectiles +30% size |
| 66 | `Cannonball` | rare | Projectiles +60% size, +30% damage |
| 67 | `Skybreaker` | epic | Projectiles +100% size, +50% damage |
| 68 | `Mass Driver` | rare | Projectiles accelerate; +1% damage per tile traveled |
| 69 | `Wormcurve` | rare | Projectiles redirect mid-flight if they would miss the nearest target (one chance per shot) |
| 70 | `Belial's Lens` | epic | Projectiles become homing after piercing the first enemy |

## Замены основной атаки (12)

**Структурно конфликтуют между собой** — занимают единственный слот
«основная атака». Если игроку выпадает второй артефакт из этой
категории, поздний перебивает раннего (ранний остаётся в стэке
затемнённым). Все модификаторы из секции «Поведение снаряда»
автоматически применяются к активной форме атаки, где это имеет смысл.

| # | Name | Tier | Effect |
|---|------|------|--------|
| 71 | `Brimstone Channel` | legendary | Заменяет атаку на charged piercing/spectral beam: hold to charge, release to fire |
| 72 | `Lightspear` | epic | Заменяет атаку на мгновенный piercing-лазер с unlimited range (no travel time) |
| 73 | `Trisagion Stream` | rare | Заменяет атаку на rapid-fire поток слабых piercing световых вспышек |
| 74 | `Reaper's Scythe` | epic | Заменяет атаку на крупные медленные piercing-косы (-50% fire rate, +200% damage) |
| 75 | `Womb Seekers` | epic | Заменяет атаку на homing fetus-снаряды, которые догоняют врагов и наносят contact damage |
| 76 | `Chain Bombs` | epic | Заменяет атаку на бросаемые бомбы с AOE-взрывом на impact |
| 77 | `Volt Coil` | epic | Заменяет атаку на chargeable lightning ring, детонирующий вокруг ближайших врагов |
| 78 | `Howitzer` | epic | Заменяет атаку на cursor-targeted ракеты, падающие сверху с большим AOE |
| 79 | `Tooth Toss` | rare | Заменяет атаку на тяжёлые медленные зубы: высокий damage, низкий fire rate |
| 80 | `Bone Knife` | epic | Заменяет атаку на возвращающийся метаемый нож; damage растёт с charge time |
| 81 | `Boulder Cannon` | rare | Заменяет атаку на тяжёлый медленный валун: огромный damage, очень низкий fire rate |
| 82 | `Lasersaw` | legendary | Заменяет атаку на непрерывную circular saw в melee-радиусе вокруг игрока |

## On-hit (25)

| # | Name | Tier | Effect |
|---|------|------|--------|
| 83 | `Smolder Tip` | common | Hits apply Burn (3 dmg/sec for 3s) |
| 84 | `Pyre Heart` | rare | Hits apply Burn (8 dmg/sec for 4s) |
| 85 | `Inferno Core` | epic | Hits apply Burn (20 dmg/sec for 5s, stacks) |
| 86 | `Frost Touch` | common | Hits slow target by 20% for 2s |
| 87 | `Glacial Edge` | rare | Hits slow target by 40% for 3s |
| 88 | `Cryo Shard` | epic | Hits have 25% chance to freeze target for 1s; frozen targets shatter on impact for AOE |
| 89 | `Venom Drip` | common | Hits apply Poison (2 dmg/sec for 5s) |
| 90 | `Toxic Bloom` | rare | Poison spreads to nearby enemies on target's death |
| 91 | `Bleed Edge` | rare | Hits apply Bleed (1% target's max HP / sec for 4s) |
| 92 | `Static Charge` | common | Every 5th hit chains lightning to 2 nearby enemies |
| 93 | `Storm Web` | rare | Every 3rd hit chains lightning to 4 nearby enemies |
| 94 | `Tesla Heart` | epic | Every hit chains lightning to 1 nearby enemy |
| 95 | `Vampiric Tip` | common | Heal 1 HP per hit |
| 96 | `Bloodthorn` | rare | Heal 2% of damage dealt |
| 97 | `Crimson Feast` | epic | Heal 5% of damage dealt (max 5 HP per hit) |
| 98 | `Pushback Round` | common | Hits knock target back slightly |
| 99 | `Tempest Slam` | rare | Hits knock target back significantly |
| 100 | `Stunlock Bolt` | rare | 10% chance to stun target for 0.5s |
| 101 | `Concussion Round` | epic | 25% chance to stun target for 1s |
| 102 | `Mark of Ruin` | rare | Hits apply a stack; at 5 stacks, your next hit deals +50% damage (consumes stacks) |
| 103 | `Hex Quill` | rare | Hits apply Curse: target takes +2% damage per stack (max 5) |
| 104 | `Fearmonger` | rare | Hits have 15% chance to make target flee for 1s |
| 105 | `Echo Strike` | epic | Hits deal a second strike for 50% damage (no on-hit triggers) |
| 106 | `Resonance` | epic | Hits create a small shockwave on the target (30% damage AOE) |
| 107 | `Soul Tap` | rare | Every 10th hit deals +500% damage |

## On-kill (15)

| # | Name | Tier | Effect |
|---|------|------|--------|
| 108 | `Necrotic Burst` | common | Kills cause a small explosion (10 dmg, small radius) |
| 109 | `Pyre Burial` | rare | Kills cause a moderate explosion (40 dmg) |
| 110 | `Apocalypse Stamp` | epic | Kills cause a large explosion (100 dmg) |
| 111 | `Soul Drain` | common | Kills heal 1 HP |
| 112 | `Heart Pluck` | rare | Kills have 10% chance to heal 5 HP |
| 113 | `Lich's Pact` | epic | Kills heal 1% max HP |
| 114 | `Killing Spree` | rare | Each kill grants +5% damage for 3s (stacks) |
| 115 | `Bloodfrenzy` | epic | Each kill grants +10% attack speed for 4s (stacks, max 10) |
| 116 | `Wraithcall` | rare | 10% chance to summon a Wraith on kill (lasts 5s, deals contact damage) |
| 117 | `Skull Crown` | epic | Every 10th kill summons a skeletal ally for the wave |
| 118 | `Lightningfall` | rare | Kills strike 2 nearby enemies with lightning (50% of your damage) |
| 119 | `Storm Reaver` | epic | Kills trigger a chain lightning (4 jumps, 100% damage) |
| 120 | `Hourglass Sand` | rare | Kills slow time briefly (0.5s, 50% slowdown) |
| 121 | `Reaper's Mark` | epic | Kills mark a random enemy: at <30% HP, they die instantly |
| 122 | `Soul Surge` | rare | Each kill grants +1% damage for the rest of the wave |

## Дефенсивы (25)

| # | Name | Tier | Effect |
|---|------|------|--------|
| 123 | `Bone Plate` | common | +10% damage reduction |
| 124 | `Iron Bulwark` | rare | +20% damage reduction |
| 125 | `Adamant Shell` | epic | +35% damage reduction |
| 126 | `Aegis Core` | common | Auto-shield blocks one hit every 8s |
| 127 | `Halo Shard` | rare | Auto-shield blocks one hit every 5s |
| 128 | `Eternal Halo` | epic | Auto-shield blocks one hit every 3s |
| 129 | `Phantom Veil` | common | 10% chance to dodge any hit |
| 130 | `Smoke Cloak` | rare | 20% chance to dodge any hit |
| 131 | `Mirage Robe` | epic | 35% chance to dodge any hit |
| 132 | `Spike Skin` | common | Reflect 20% contact damage to attacker |
| 133 | `Blade Shroud` | rare | Reflect 50% damage to attacker |
| 134 | `Razor Aura` | epic | Reflect 100% damage; deal 5 dmg/sec to enemies in melee range |
| 135 | `Stalwart Heart` | rare | Above 80% HP: -50% damage taken |
| 136 | `Last Light` | rare | Below 25% HP: -50% damage taken |
| 137 | `Glass Will` | epic | Negate the next hit that would kill you (CD 30s) |
| 138 | `Revival Core` | legendary | On lethal damage, heal to 50% HP instead (once per run) |
| 139 | `Stoneblood` | rare | Taking damage grants 50% damage reduction for 1s (CD 4s) |
| 140 | `Lifeline` | rare | When dropped below 30% HP, gain a 3s 50%-damage shield (CD 20s) |
| 141 | `Etheric Pulse` | common | 0.5s of i-frames after each hit taken |
| 142 | `Counter Pulse` | rare | Taking damage triggers a small AOE shockwave (30 dmg) |
| 143 | `Counter Bloom` | epic | Taking damage triggers a moderate AOE shockwave (80 dmg) |
| 144 | `Chillshroud` | rare | Enemies in melee range are slowed by 30% |
| 145 | `Sapping Aura` | rare | Enemies in melee range take 5 dmg/sec |
| 146 | `Repulsion Field` | epic | Continuously knocks back enemies in melee range |
| 147 | `Phase Drift` | epic | Enemy contact deals no damage (projectiles still hit) |

## Healing (10)

| # | Name | Tier | Effect |
|---|------|------|--------|
| 148 | `Bloodroot` | common | Regenerate 0.5 HP/sec |
| 149 | `Vinetwine` | rare | Regenerate 1.5 HP/sec |
| 150 | `Worldroot` | epic | Regenerate 1% max HP/sec |
| 151 | `Soul Knit` | rare | Heal 25% max HP at the start of each wave |
| 152 | `Healer's Mark` | common | Heal 5 HP at the start of each wave |
| 153 | `Last Drop` | rare | Heal 20% missing HP every 10s |
| 154 | `Vampire's Pact` | epic | Heal 3% of damage dealt (stacks with lifesteal) |
| 155 | `Heart Bloom` | rare | Killing 5 enemies in 3s heals 10 HP |
| 156 | `Sanguine Tide` | epic | When you drop below 50% HP, heal 30 HP (CD 30s) |
| 157 | `Pulse of Life` | rare | All healing effects are +50% effective |

## Exotic / unique (43)

| # | Name | Tier | Effect |
|---|------|------|--------|
| 158 | `Orbital Shard` | common | 1 orb circles you, dealing 5 dmg on contact |
| 159 | `Orbital Cluster` | rare | 2 orbs circle you (faster, more damage) |
| 160 | `Orbital Constellation` | epic | 3 orbs circle you in a wider radius |
| 161 | `Orbital Genesis` | legendary | 5 orbs that also fire homing shots periodically |
| 162 | `Sentry Spark` | common | 1 stationary turret spawns at start of wave |
| 163 | `Sentry Wing` | rare | 2 stationary turrets spawn at start of wave |
| 164 | `Sentry Crown` | epic | 3 turrets that share your stat upgrades |
| 165 | `Pulse Beacon` | common | Periodic AOE pulse around you (every 3s, 15 dmg) |
| 166 | `Resonance Beacon` | rare | Periodic AOE pulse (every 2s, 30 dmg) |
| 167 | `World Beacon` | epic | Continuous damaging aura (5 dmg/sec, large radius) |
| 168 | `Skybreaker Strike` | epic | Every 5s, lightning strikes a random enemy (100 dmg) |
| 169 | `Meteor Vow` | legendary | Every 8s, a meteor falls on the strongest enemy (300 dmg AOE) |
| 170 | `Brimstone Imp` | rare | A familiar charges and fires a periodic mini-beam at enemies |
| 171 | `Bone Familiar` | rare | A skeletal familiar runs alongside you, attacking in melee |
| 172 | `Wisp Choir` | epic | +1 wisp familiar per 10 kills (caps at 5, fires shots) |
| 173 | `Glacial Drone` | epic | A familiar fires icicles; frozen enemies shatter for AOE damage |
| 174 | `Mine Layer` | common | Drops a proximity mine every 2s |
| 175 | `Bombardier` | rare | Drops a proximity bomb every 1s |
| 176 | `Trapped Cards` | epic | Each shot has 10% chance to drop a trap card behind you |
| 177 | `Gravity Well` | epic | Every 4s, creates a gravity well that pulls and damages enemies |
| 178 | `Cosmic Rift` | legendary | Continuously pulls enemies toward a roving point on the map |
| 179 | `Hourglass Sigil` | rare | Time slows for 1s every 10s (auto) |
| 180 | `Fractal Tear` | epic | Every 5th projectile fires as a ricocheting beam |
| 181 | `Echo Volley` | epic | Each shot has 20% chance to fire twice |
| 182 | `Doppelganger` | legendary | A spectral copy mimics your shots (50% damage) |
| 183 | `Time Lock` | epic | Every 5s, freezes all enemies for 1s |
| 184 | `Curse Mark` | rare | Every 8s, marks the strongest enemy: +100% damage taken |
| 185 | `Fire Trail` | common | Leaves a fire patch behind you while moving |
| 186 | `Frost Trail` | rare | Leaves a slowing frost patch behind you while moving |
| 187 | `Spark Trail` | rare | Leaves an arcing lightning patch behind you while moving |
| 188 | `Voltaic Footfall` | epic | Each step deals AOE damage in a small radius |
| 189 | `Stomp Aftermath` | epic | Standing still 1+ seconds creates a damaging shockwave |
| 190 | `Whirlwind Step` | rare | Moving 3+ seconds spawns wind-blade projectiles |
| 191 | `Eclipse` | legendary | Every 30s, all enemies on screen take 50% of their max HP as damage |
| 192 | `Time Slip` | legendary | Each wave starts with 5s of frozen time and invulnerability |
| 193 | `Pyromaniac's Pact` | epic | Immune to your own AOE damage; heal for 50% of self-AOE damage that would have hit you |
| 194 | `Titan Bloom` | legendary | Once per run, become giant + invulnerable + +500% damage for 15s (auto-triggers below 30% HP) |
| 195 | `Charge Coil` | rare | Holding fire 2s charges a powerful super-shot |
| 196 | `Deflector Field` | epic | A field around you reflects enemy projectiles back |
| 197 | `Bullet Time Aura` | rare | Enemy projectiles within close range slow by 50% |
| 198 | `Berserker Crown` | epic | Each hit taken: +5% damage for the wave (max +50%) |
| 199 | `Cataclysm Engine` | legendary | Every 60s, deal 1000 damage to all enemies on screen |
| 200 | `Worldhammer` | legendary | Each shot has 1% chance to deal 50× damage |
