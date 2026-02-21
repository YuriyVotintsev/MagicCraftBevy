# Ревью: calc-spec.md — Единая система выражений

## Общая оценка

Спека решает реальные проблемы: дублирование двух типов выражений (`stats::Expression` и `blueprints::ScalarExpr`), повторяющиеся формулы в RON-файлах абилок, и громоздкий `calculators` + `AggregationType`. Предложенная архитектура чистая — направление зависимостей `expr → ничего`, `stats → expr`, `blueprints → expr, stats` корректно. Объём изменений значительный, но каждый шаг имеет ясную мотивацию.

---

## Сильные стороны

1. **Единый `ScalarExpr`** — объединение 11 вариантов `stats::Expression` и 17 вариантов `blueprints::ScalarExpr` в один тип устраняет дублирование арифметики и stat-доступа. Добавление `Clamp` в единый тип (сейчас есть только в stats) — логично.

2. **`AtomParser` trait** — ограничение контекста на этапе парсинга, а не runtime. `StatAtomParser` отвергает `index`/`count`/`caster_pos` с ошибкой — ловит баги рано. Сейчас такой защиты нет: stat-формулы используют отдельный тип `Expression`, который физически не может содержать spawn-контекст, но это через два разных типа, а не через валидацию.

3. **`StatProvider` trait + `EvalCtx`** — абстракция eval-контекста через trait вместо прямой зависимости от `ComputedStats`. `stat_only()` конструктор с нулями — прагматичное решение, безопасное при наличии `StatAtomParser`.

4. **`Formula("...")` вместо `Standard`/`Custom`/`calculators`** — радикальное упрощение. Один enum `StatEvalKind { Sum, Product, Formula }` вместо `AggregationType` (4 варианта) + отдельная секция `calculators`. `depends_on` автоматически из дерева выражений — убирает источник рассинхрона.

5. **Текстовая подстановка `calc()`** — простое и предсказуемое решение. Раскрытие до парсинга означает: парсер видит только арифметику и `stat()`, runtime работает с плоским деревом без обращений к `CalcRegistry`. Нет overhead в runtime.

6. **Рекурсивные шаблоны** — `physical_damage` → `flat_increased_more` — двухуровневая абстракция: domain-шаблоны (physical_damage, projectile_speed) вызывают generic-шаблоны (flat_increased_more). Позволяет менять формулу в одном месте для всех абилок.

7. **Жизненный цикл (§4)** — чётко описанный pipeline от RON-строки до f32. Два пути (blueprint и stat) хорошо визуализированы.

---

## Проблемы и вопросы

Ошибки при загрузке (resolve, expand, парсинг, циклы зависимостей) ловятся тестами и паниками при старте — это нормальное поведение, не требует специальной обработки.

Runtime-проблем в предложенной архитектуре не видно: после загрузки все выражения — плоские деревья из арифметики и `StatId`, eval — чистый match без fallible операций (деление на ноль обрабатывается через `safe_div`).

---

## Рекомендации по порядку реализации

Спека не описывает порядок имплементации. Предлагаю:

1. **`src/expr/` модуль** — создать с нуля: `ScalarExpr`/`ScalarExprRaw`, `VecExpr`/`VecExprRaw`, `EntityExpr`, `StatId`, `StatProvider`, `EvalCtx`, `parser.rs`, `calc.rs`. Можно покрыть unit-тестами изолированно.

2. **`CalcRegistry`** — загрузка `calcs.ron`, текстовая подстановка, тесты expand().

3. **Миграция `stats/`** — `AggregationType` → `StatEvalKind`, удалить `stats::Expression`, удалить `calculators`, формулы инлайн. `impl StatProvider for ComputedStats`.

4. **Миграция `blueprints/`** — переключить на `expr::ScalarExpr`, `BlueprintAtomParser`, конструирование `EvalCtx` из `SpawnSource`.

5. **RON-файлы** — обновить `config.stats.ron` и ability RON файлы на `calc()` синтаксис.

6. **`_base` → `_flat`** — отдельный механический проход.

7. **Удаление старого кода** — `stats/expression.rs`, старый парсер (если не переиспользуется), `AggregationType`, `calculators`.

---

## Вердикт

Спека готова к реализации. Word boundary подстановка зафиксирована в спеке, `ExprFamily` удаляется полностью. Runtime-проблем не обнаружено.
