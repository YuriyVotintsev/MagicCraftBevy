# Code Review модуля Artifacts

## Major

### 1. `AvailableArtifacts` не обновляется после загрузки
**Файл:** `src/artifacts/resources.rs`

`AvailableArtifacts` устанавливается один раз при загрузке и больше не модифицируется. Нет механизма исключения уже купленных артефактов из пула, поэтому один и тот же артефакт может появляться в магазине несколько раз и стакаться.

**Исправление:** Добавить методы `remove()` / `add()`, или как минимум задокументировать намеренное поведение.

---

## Minor

### 2. `ShopOfferings::remove()` паникует при невалидном индексе
**Файл:** `src/artifacts/resources.rs`

`Vec::remove()` паникует если `index >= len`. Хотя `handle_buy` проверяет `event.index >= offerings.len()` перед вызовом, ничто не мешает прямому вызову `ShopOfferings::remove()` с невалидным индексом из другого места.

```rust
pub fn remove(&mut self, index: usize) -> Entity {
    self.0.remove(index)
}
```

**Исправление:** Возвращать `Option<Entity>` и использовать `self.0.get(index)` + `self.0.remove(index)`, либо валидировать индекс внутри метода.

---

### 3. `ArtifactRegistry::get_id()` — мёртвый код
**Файл:** `src/artifacts/registry.rs`

Метод `get_id(&self, name: &str) -> Option<ArtifactId>` нигде не вызывается. HashMap `name_to_id` заполняется, но никогда не запрашивается.

**Исправление:** Удалить метод и поле `name_to_id`, либо добавить `#[allow(dead_code)]` если планируется использовать в будущем.

---

### 4. `ArtifactDef::sell_price()` — усечение при целочисленном делении
**Файл:** `src/artifacts/types.rs`

```rust
pub fn sell_price(&self, percent: u32) -> u32 {
    self.price * percent / 100
}
```

При `price = 3` и `percent = 50` результат `3 * 50 / 100 = 1` (усечено с 1.5). При `price = 1` и `percent = 50` результат `0`. Целочисленное усечение означает, что дешёвые артефакты продаются за ничего.

**Исправление:** Если нужно округление вверх — использовать `(self.price * percent + 99) / 100` или `(self.price * percent).div_ceil(100)`.

---

### 5. `PlayerArtifacts` default хардкодит 5 слотов вместо значения из `GameBalance`
**Файл:** `src/artifacts/resources.rs`

`Default` impl хардкодит 5 слотов, хотя реальное количество берётся из `balance.shop.artifact_slots`. Default используется кратковременно до перезаписи в `reset_artifacts`, но это ловушка для поддержки — если `artifact_slots` изменится на 3, default всё равно создаст 5.

**Исправление:** Использовать пустой Vec в `Default` (`slots: Vec::new()`), т.к. `reset_artifacts` всегда перезаписывает его сразу.

---

### 6. `RerollCost::default()` хардкодит 1 вместо значения из balance
**Файл:** `src/artifacts/resources.rs`

Та же проблема что и #5. Default — `1`, но реальное значение берётся из `balance.shop.base_reroll_cost`. Если base cost в balance изменится на 2, default будет неверным до запуска `reset_artifacts`.

**Исправление:** Поставить default `0` или задокументировать что default немедленно перезаписывается.

---

### 7. `ArtifactDefRaw` имеет поле `id`, которое не сохраняется в `ArtifactDef`
**Файл:** `src/artifacts/types.rs`

`ArtifactDefRaw` имеет `pub id: String`, который используется как ключ при регистрации (`artifact_registry.register(&raw.id, def)`), но `ArtifactDef` не хранит `id`. После загрузки нет способа получить строковый ID артефакта из его `ArtifactDef`, что затрудняет отладку и отображение.

**Исправление:** Сохранять `id` в `ArtifactDef` если нужно для логирования/отладки, либо принять как намеренное решение если числовых ID достаточно.

---

### 8. `rand::rng()` создаётся заново при каждом реролле
**Файл:** `src/artifacts/systems.rs`

Новый RNG создаётся из энтропии при каждом вызове `reroll_offerings`. Хотя для реролов в магазине (редкая операция) это не проблема производительности, лучше использовать сидированный/персистентный RNG для воспроизводимости и тестируемости.

**Исправление:** Хранить `RngResource` и передавать его, либо использовать Bevy `GlobalRng` если доступен.
