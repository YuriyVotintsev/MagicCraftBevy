# Code Review модуля Artifacts

## Major

### 4. `handle_reroll` не списывает деньги
**Файл:** `src/artifacts/systems.rs:45-55`

Система `handle_reroll` принимает `RerollRequest` и регенерирует offerings, но никогда не проверяет и не списывает стоимость реролла. Списание денег и увеличение стоимости делаются в `ui/shop.rs:reroll_system` (строка 357-358). Бизнес-логика разделена: деньги — в UI слое, артефакты — в доменном. Если кто-то отправит `RerollRequest` не из UI (тест, дебаг-консоль, другая система), реролл будет бесплатным.

```rust
pub fn handle_reroll(
    // ...нет параметра PlayerMoney или RerollCost
) {
    for _ in events.read() {
        reroll_offerings(&mut commands, &mut offerings, &available, balance.shop.offerings_count);
    }
}
```

**Исправление:** Перенести проверку денег + списание + увеличение стоимости в сам `handle_reroll`. UI должен только отправлять сообщение; обработчик должен владеть полной транзакцией.

---

### 5. `reroll_cost` сбрасывается в двух местах
**Файлы:** `src/artifacts/mod.rs:46` и `src/ui/shop.rs:161`

`reroll_cost.reset_to(balance.shop.base_reroll_cost)` вызывается в `reset_artifacts` (при `OnEnter(GameState::Playing)`) и снова в `spawn_shop` (при `OnEnter(WavePhase::Shop)`). Сброс на уровне Shop означает, что каждую волну стоимость возвращается к базовой. Но тогда сброс на уровне Playing избыточен. Если замысел — "стоимость сохраняется между волнами, но сбрасывается за ран", то баг в Shop-level сбросе. Если замысел — "сбрасывать каждую фазу магазина", то Playing-level сброс — мёртвый код.

**Исправление:** Определиться с одной точкой сброса и убрать вторую.

---

### 6. `AvailableArtifacts` не обновляется после загрузки
**Файл:** `src/artifacts/resources.rs:79-89`

`AvailableArtifacts` устанавливается один раз при загрузке и больше не модифицируется. Нет механизма исключения уже купленных артефактов из пула, поэтому один и тот же артефакт может появляться в магазине несколько раз и стакаться. См. также проблему #3.

**Исправление:** Добавить методы `remove()` / `add()`, или как минимум задокументировать намеренное поведение.

---

### 7. `ShopOfferings::remove()` паникует при невалидном индексе
**Файл:** `src/artifacts/resources.rs:59-61`

`Vec::remove()` паникует если `index >= len`. Хотя `handle_buy` проверяет `event.index >= offerings.len()` перед вызовом, ничто не мешает прямому вызову `ShopOfferings::remove()` с невалидным индексом из другого места.

```rust
pub fn remove(&mut self, index: usize) -> Entity {
    self.0.remove(index)
}
```

**Исправление:** Возвращать `Option<Entity>` и использовать `self.0.get(index)` + `self.0.remove(index)`, либо валидировать индекс внутри метода.

---

### 8. `handle_buy`: `StatRange::Range` всегда берёт середину диапазона
**Файл:** `src/artifacts/systems.rs:89-91`

Когда артефакт имеет модификаторы с `Range`, применяемое значение всегда `(min + max) / 2.0`. Такой детерминизм делает `Range` функционально идентичным `Fixed` со средним значением — вариант `Range` бесполезен.

```rust
StatRange::Range { stat, min, max } => {
    modifiers.add(*stat, (*min + *max) / 2.0, Some(artifact_entity));
}
```

Та же логика с серединой есть в тултипе (`src/ui/artifact_tooltip.rs:102`), так что отображение совпадает. Но это обесценивает смысл `Range`.

**Исправление:** Если Range должен давать случайное значение — использовать `rng.gen_range(*min..=*max)`. Если всегда середина — использовать `Fixed` в RON-данных и удалить мёртвую ветку `Range`.

---

### 9. Цвет тултипа определяется только по первому стату
**Файл:** `src/ui/artifact_tooltip.rs:106`

Когда модификатор имеет несколько статов (например, `+5 damage, -2 speed`), цвет ВСЕХ строк определяется только по значению первого стата. Если первый стат положительный — все строки зелёные, даже если последующие отрицательные.

```rust
let value = pairs.first().map(|(_, v)| *v).unwrap_or(0.0);
let color = if value > 0.0 { POSITIVE_COLOR } else { NEGATIVE_COLOR };
```

**Исправление:** Определять цвет для каждой строки/стата отдельно, а не для всего модификатора.

---

### 10. `handle_buy`/`handle_sell`/`handle_reroll` работают каждый фрейм без гейтинга по состоянию
**Файл:** `src/artifacts/mod.rs:32`

Эти системы зарегистрированы в `Update` без `.run_if(in_state(WavePhase::Shop))`:

```rust
.add_systems(Update, (systems::handle_reroll, systems::handle_buy, systems::handle_sell));
```

Поскольку они используют `MessageReader`, вне фазы Shop событий не будет (они пишутся только из Shop UI). Но системы всё равно запускаются (обращаются к ресурсам, итерируют пустые event reader'ы) каждый фрейм во время боя. Это лишняя работа.

**Исправление:** Добавить `.run_if(in_state(WavePhase::Shop))` чтобы не запускать эти системы во время боя.

---

## Minor

### 11. `ArtifactRegistry::get_id()` — мёртвый код
**Файл:** `src/artifacts/registry.rs:25-27`

Метод `get_id(&self, name: &str) -> Option<ArtifactId>` нигде не вызывается. HashMap `name_to_id` заполняется, но никогда не запрашивается.

```rust
pub fn get_id(&self, name: &str) -> Option<ArtifactId> {
    self.name_to_id.get(name).copied()
}
```

**Исправление:** Удалить метод и поле `name_to_id`, либо добавить `#[allow(dead_code)]` если планируется использовать в будущем.

---

### 12. `ArtifactDef::sell_price()` — усечение при целочисленном делении
**Файл:** `src/artifacts/types.rs:18-20`

```rust
pub fn sell_price(&self, percent: u32) -> u32 {
    self.price * percent / 100
}
```

При `price = 3` и `percent = 50` результат `3 * 50 / 100 = 1` (усечено с 1.5). При `price = 1` и `percent = 50` результат `0`. Целочисленное усечение означает, что дешёвые артефакты продаются за ничего.

**Исправление:** Если нужно округление вверх — использовать `(self.price * percent + 99) / 100` или `(self.price * percent).div_ceil(100)`.

---

### 13. `AvailableArtifacts::to_vec()` лишнее клонирование
**Файл:** `src/artifacts/resources.rs:86-88`

Каждый вызов `reroll_offerings` запускает `available.to_vec()`, который клонирует весь внутренний Vec. Вызывается при каждом входе в магазин и каждом реролле.

```rust
pub fn to_vec(&self) -> Vec<ArtifactId> {
    self.0.clone()
}
```

**Исправление:** Предоставить `as_slice() -> &[ArtifactId]`, а копию создавать только когда нужно шаффлить. Либо шаффлить индексы вместо клонирования данных.

---

### 14. `PlayerArtifacts` default хардкодит 5 слотов вместо значения из `GameBalance`
**Файл:** `src/artifacts/resources.rs:10-15`

`Default` impl хардкодит 5 слотов, хотя реальное количество берётся из `balance.shop.artifact_slots`. Default используется кратковременно до перезаписи в `reset_artifacts`, но это ловушка для поддержки — если `artifact_slots` изменится на 3, default всё равно создаст 5.

```rust
impl Default for PlayerArtifacts {
    fn default() -> Self {
        Self {
            slots: vec![None; 5],
        }
    }
}
```

**Исправление:** Использовать пустой Vec в `Default` (`slots: Vec::new()`), т.к. `reset_artifacts` всегда перезаписывает его сразу.

---

### 15. `RerollCost::default()` хардкодит 1 вместо значения из balance
**Файл:** `src/artifacts/resources.rs:108-112`

Та же проблема что и #14. Default — `1`, но реальное значение берётся из `balance.shop.base_reroll_cost`. Если base cost в balance изменится на 2, default будет неверным до запуска `reset_artifacts`.

```rust
impl Default for RerollCost {
    fn default() -> Self {
        Self(1)
    }
}
```

**Исправление:** Поставить default `0` или задокументировать что default немедленно перезаписывается.

---

### 16. `ArtifactDefRaw` имеет поле `id`, которое не сохраняется в `ArtifactDef`
**Файл:** `src/artifacts/types.rs:25,33`

`ArtifactDefRaw` имеет `pub id: String`, который используется как ключ при регистрации (`artifact_registry.register(&raw.id, def)`), но `ArtifactDef` не хранит `id`. После загрузки нет способа получить строковый ID артефакта из его `ArtifactDef`, что затрудняет отладку и отображение.

**Исправление:** Сохранять `id` в `ArtifactDef` если нужно для логирования/отладки, либо принять как намеренное решение если числовых ID достаточно.

---

### 17. `rand::rng()` создаётся заново при каждом реролле
**Файл:** `src/artifacts/systems.rs:34`

Новый RNG создаётся из энтропии при каждом вызове `reroll_offerings`. Хотя для реролов в магазине (редкая операция) это не проблема производительности, лучше использовать сидированный/персистентный RNG для воспроизводимости и тестируемости.

```rust
let mut rng = rand::rng();
```

**Исправление:** Хранить `RngResource` и передавать его, либо использовать Bevy `GlobalRng` если доступен.
