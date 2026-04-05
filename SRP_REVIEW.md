# SRP Review — Magic Craft Bevy

Дата: 2026-04-04
Проанализировано: ~100 .rs файлов

---

## Критичные нарушения

### 1. `src/loading/systems.rs` (303 строки)

**Ответственности:**
- Загрузка и инициализация stats (registry, calculators, formulas)
- Загрузка и регистрация blueprints
- Загрузка и построение skill tree
- Загрузка и инициализация particle configs

**Проблемные функции:**
- `check_stats_loaded()` (строки ~49-147): читает asset, создаёт registry, строит calculators, парсит формулы, собирает зависимости
- `check_content_loaded()` (строки ~149-281): загружает папки, регистрирует blueprints, строит skill tree, создаёт particle registry

**Рекомендация:** Разделить на отдельные системы загрузки по домену: StatsLoader, BlueprintLoader, SkillTreeLoader, ParticleLoader. Каждая система слушает свой этап загрузки.

---

### 2. `src/scenario/runner.rs` (399 строк)

**Ответственности:**
- Состояние сценария (таймеры, шаги, assertion tracking)
- Диспатч действий (движение, ввод, ожидание, логирование)
- Проверка assertions (entity counts, stat values, comparisons)
- Сериализация state dump
- Симуляция клавиатурного ввода
- Авто-скип меню

**Проблемные функции:**
- `scenario_system()` (строки ~48-124): оркестрирует выполнение шагов, диспатч действий, проверку assertions
- `execute_action()` (строки ~126-193): огромный match на типы действий
- `check_assertion()` (строки ~195-278): match на типы assertions с разной логикой проверки
- `dump_state()` (строки ~297-321): сериализация состояния — отдельная ответственность

**Рекомендация:** Разделить на ScenarioRunner (оркестрация), ActionExecutor (диспатч действий), AssertionChecker (проверки), StateDumper (сериализация), InputSimulator (клавиатура).

---

### 3. `src/arena.rs` (394 строки)

**Ответственности:**
- Setup стен (физика)
- Генерация floor mesh
- Setup и управление камерой
- Спавн врагов (выбор позиции, выбор blueprint, создание summoning circle)
- Скейлинг врагов по времени
- Обновление floor mesh при изменении камеры

**Проблемные функции:**
- `spawn_enemies()` (строки ~188-277): 8 подзадач — wave state, позиция игрока, границы арены, рандом позиции, выбор blueprint, размер круга, спавн entity, обновление wave state
- `setup_arena()`: стены + пол + камера в одном месте

**Рекомендация:** Вынести спавн врагов в отдельный модуль `enemy_spawning.rs`. Камеру — в `camera.rs`. Оставить в `arena.rs` только стены и пол.

---

### 4. `src/particles.rs` (399 строк)

**Ответственности:**
- Загрузка и наследование конфигов (parent field resolution)
- Registry management (хранение, lookup)
- Material cache management
- Emission логика (burst vs continuous, rate timing)
- Spawn shape handling (Point, Circle, override)
- Физика и lifetime отдельных частиц
- Lifecycle emitter-сущностей (start, stop, drain, despawn)

**Проблемные функции:**
- `ParticleRegistry::resolve_all()` / `resolve_one()` (строки ~101-144): рекурсивное наследование + visited tracking + defaults — это отдельный алгоритм
- `emit_particles()` (строки ~240-299): burst vs continuous rate + emitter lifecycle + spawn delegation + config lookup
- `spawn_burst()` (строки ~301-353): shape override selection + random generation + velocity + entity spawning

**Рекомендация:** Разделить на: `particle_config.rs` (загрузка, наследование, registry), `particle_emitter.rs` (emission логика, lifecycle), `particle_spawn.rs` (создание отдельных частиц, физика).

---

### 5. `src/ui/skill_tree_view.rs` (500+ строк)

**Ответственности:**
- Отрисовка дерева (ноды, грани, текст)
- Обработка кликов и hover
- Camera pan и zoom
- Обновление визуального состояния нод
- Tooltip display
- Overlay UI текст

**Проблемные функции:**
- `skill_tree_pan_zoom()` (строки ~351-391): input detection + camera transformation
- `skill_tree_click()` (строки ~393-445): mouse input + collision detection + event emission — 3 ответственности
- `update_node_visuals()` (строки ~447-481): обновление и edge, и node визуалов
- `skill_tree_hover()` (строки ~483+): tooltip spawning + viewport calculations + entity despawning

**Рекомендация:** Разделить на: `skill_tree_input.rs` (клики, hover), `skill_tree_rendering.rs` (ноды, грани), `skill_tree_camera.rs` (pan/zoom), `skill_tree_tooltips.rs`.

---

### 6. `src/run.rs` (252 строки)

**Ответственности:**
- Трекинг состояния рана (elapsed, attempt count)
- Drain HP игрока по времени
- Детекция смерти и начало death sequence
- 3-фазная анимация смерти (Landing → Shrink → Done)
- Shrink анимация всех сущностей
- Спавн particle эффектов
- Переход в GameOver state

**Проблемные функции:**
- `check_run_end()` (строки ~97-134): death detection + movement lock + resource insertion + shrink initialization — 4 задачи
- `player_death_sequence()` (строки ~176-247): landing detection + child animation check + particle spawning + scale animation + phase transitions — 5+ задач
- `animate_shrink_to_zero()` (строки ~156-172): animation logic + entity despawning

**Рекомендация:** Вынести death sequence в отдельный модуль `death_sequence.rs` с системами по фазам. Run state tracking оставить в `run.rs`.

---

### 7. `src/summoning.rs` (254 строки)

**Ответственности:**
- State machine суммона (3 фазы)
- Circle scaling и визуальные эффекты
- Enemy rising анимация
- Particle emission management
- Blueprint entity spawning
- Initial state activation

**Проблемная функция:**
- `animate_summoning()` (строки ~105-182): один match на 3 фазы (CircleGrow, EnemyRise, CircleShrink), каждая с совершенно разной логикой — scaling, particle emission, blueprint spawning, wave state updates

**Рекомендация:** Разделить на 3 системы по фазам с `run_if` по текущей фазе, либо оставить в одном файле но вынести каждую фазу в отдельную функцию.

---

## Высокие нарушения

### 8. `src/blueprints/spawn.rs` (260 строк)

**Ответственности:** entity spawning + component insertion + FSM initialization + stat setup + modifier application

**Проблемные функции:**
- `spawn_root()` (строки ~82-124): entity creation + component insertion + FSM initialization
- `init_identity()` (строки ~161-205): identity setup + modifier application
- `insert_components()` (строки ~216-230): component insertion + recalc tracking

**Рекомендация:** Разделить на EntityFactory, ComponentInserter, FsmInitializer.

---

### 9. `src/expr/parser.rs` (557 строк)

**Ответственности:** лексер (tokenization) + парсер (expression tree building) + 20+ parse methods для разных типов + type validation

**Проблемные места:**
- `lex()` (строки ~17-90): полный lexer
- `parse_ident_expr()` (строки ~256-287): switch на 13+ типов выражений
- Строки ~307-457: 20+ отдельных parse methods

**Рекомендация:** Вынести lexer в отдельный файл `expr/lexer.rs`. Parser оставить, но сгруппировать parse methods по категориям (functions, contexts, operators).

---

### 10. `src/blueprints/components/common/sprite.rs` (262 строки)

**Ответственности:** color deserialization + shape enum handling + mesh resource setup + material creation + 4 почти одинаковых spawn_*_visuals функции

**Проблемные места:**
- Строки ~9-55: Color parsing и serialization (отдельная ответственность)
- Строки ~176-261: 4 похожих функции `spawn_*_visuals` с дублированием кода

**Рекомендация:** Вынести color parsing в `palette.rs` или `color_utils.rs`. Объединить 4 spawn функции в одну generic.

---

### 11. `src/stats/display.rs` (258 строк)

**Ответственности:** format string parsing + template extraction + value formatting + color selection + registry initialization

**Проблемные функции:**
- `parse_format_string()` (строки ~72-121): string parsing + bracket matching + template parsing
- `StatDisplayRegistry::new()` (строки ~148-195): parsing + format building + fallback creation

**Рекомендация:** Вынести FormatStringParser отдельно от registry.

---

### 12. `src/ui/stat_line_builder.rs` (380 строк)

**Ответственности:** 3 render mode (Fixed, Range, Diff) в одном `collect_segments()` + UI spawning + color logic

**Проблемная функция:**
- `collect_segments()` (строки ~77-148): огромный match по Fixed/Range/Diff — по сути 3 разных алгоритма в одном месте

**Рекомендация:** Создать отдельные builder-ы по render mode или хотя бы вынести каждый mode в отдельную функцию.

---

### 13. `src/blueprints/state.rs` (101 строка)

**Ответственности:** state comparison + component removal + component insertion + recalc storage management + debug logging

**Проблемная функция:**
- `state_transition_system()` (строки ~24-100): 5 ответственностей в одной функции

**Рекомендация:** Вынести component lifecycle (insert/remove) в отдельный helper.

---

### 14. `src/blueprints/activation.rs` (126 строк)

**Ответственности:** blueprint activation + cooldown management + spawn tracking + wave-specific enemy tagging

**Проблемное место:**
- Строки ~75-87: wave enemy insertion смешано с основной spawn логикой

**Рекомендация:** Вынести wave-specific логику (WaveEnemy tag) из activation system.

---

### 15. `src/health_material.rs` (128 строк)

**Ответственности:** enemy sprite identification + UV coordinate calculation + material instantiation + material linking + health updates

**Проблемная функция:**
- `apply_health_material()` (строки ~46-107): 5 подзадач — идентификация, поиск child sprite, UV расчёт, создание material, замена component

**Рекомендация:** Вынести UV calculation и material factory в отдельные функции.

---

## Средние нарушения

### 16. `src/skill_tree/generation.rs` (263 строки)

Содержит 4 разных алгоритма: Poisson disk sampling, Delaunay triangulation, edge pruning, connectivity check. Каждый — отдельная ответственность, но все нужны только для генерации. Допустимо оставить в одном файле, но вынести каждый алгоритм в отдельную функцию (уже частично сделано).

### 17. `src/blueprints/tests.rs` (227 строк)

File I/O + parsing + validation + reference tracking — тестовый код, допустимо.

### 18. `src/stats/calculators.rs` (163 строки)

Dependency graph + topological sort + calculation execution + dirty invalidation. Связанные ответственности, но `topological_sort()` — отдельный алгоритм, который можно вынести.

### 19. `src/ui/dev_menu.rs` (194 строки)

Camera angle slider + UI spawning + pause/unpause + cheat money. Дев-инструмент, допустимо.

### 20. `src/ui/pause_menu.rs` (148 строк)

Input handling + game state transitions + virtual time pausing + button colors. Типичный UI модуль, допустимо для небольшого размера.

### 21. `src/coin.rs` (152 строки)

Move easing + position updates + collection detection. Минимальное смешение, допустимо.

---

## Дублирование кода

| Что | Где | Рекомендация |
|-----|-----|-------------|
| `rotate_vec2()` | `boomerang.rs`, `straight.rs` | Вынести в общий utils |
| Faction-based layer selection | `destroy_enemy_projectiles.rs`, `find_nearest_enemy.rs`, `find_random_enemy.rs`, `melee_strike.rs`, `on_area.rs`, `on_collision.rs` | Создать helper `enemy_layer_for_faction()` |
| 4x `spawn_*_visuals()` | `sprite.rs` строки 176-261 | Объединить в одну generic функцию |
| `find_ron_files()` | `blueprints/tests.rs`, `validation_tests.rs` | Вынести в общий test utils |

---

## Файлы без нарушений (примеры хорошего SRP)

- `src/game_state.rs` — только enum и state transitions
- `src/schedule.rs` — только SystemSet определения
- `src/faction.rs` — только Faction enum
- `src/coord.rs` — только coordinate utilities
- `src/common/lifecycle.rs` — только entity lifecycle on owner death
- `src/physics/layers.rs` — только layer definitions
- `src/stats/dirty_stats.rs` — только dirty tracking
- `src/stats/stat_id.rs` — только registry и lookup
- `src/stats/modifier_def.rs` — только data definition
- `src/blueprints/cleanup.rs` — только cleanup orphaned entities
- `src/blueprints/recalc.rs` — только recalculation on stats change
- Все blueprint component файлы < 60 строк — как правило чистый SRP

---

## Итог

- **Критичных нарушений:** 7 файлов (loading/systems, scenario/runner, arena, particles, skill_tree_view, run, summoning)
- **Высоких:** 8 файлов
- **Средних:** 6 файлов
- **Дублирование:** 4 паттерна
- **Чистых:** ~75 файлов (~75% проекта)

Основная тенденция: нарушения концентрируются в "оркестрирующих" файлах (loading, arena, run) и в UI. Blueprint components в целом хорошо следуют SRP благодаря `#[blueprint_component]` макросу.
