#![no_std]
#![no_main]

// Импорты и точка входа в программу
use cortex_m as _;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4::stm32f411;

// Красный светодиод - PA6  (D12)
// Желтый светоиод - PB6  (D10)
// Зелёеый светодиод - PA9  (D8)
// Перавя кнопка - PB3  (D3), внешняя кнопка без резистора, внутренний pull-up, нажата = LOW. Включает/выключает.
// Вторая кнопка - PC13 (B1), встроенная кнопка, внутренний pull-up, нажата = LOW. Только меняет режим первой кнопки

// Реализация управления светодиодами через PAC
fn apply_leds(dp: &stm32f411::Peripherals, lit_count: u8) {
    dp.GPIOA.odr().modify(|_, w| w.odr6().bit(lit_count >= 1)); // PA6 (красный): включить/выключить одним вызовом
    dp.GPIOB.odr().modify(|_, w| w.odr6().bit(lit_count >= 2)); // PB6 (жёлтый)
    dp.GPIOA.odr().modify(|_, w| w.odr9().bit(lit_count >= 3)); // PA9 (зелёный)
}

// Реализация задержки
fn helper_delay_ms(ms: u32) { // Читаем временную переменную, чтобы создать задержку
    for _ in 0..ms {
        for _ in 0..4000 {
            unsafe { // Через ключевое слово unsafe отключаем проверку безопасности компилятором
                core::ptr::read_volatile(&0u32 as *const u32); // Чтобы компилятор не удалял пустой цикл, явно прописываем обратное
            }
        }
    }
}

// Точка входа
#[entry]
fn main() -> ! { // -> ! означает, что программа работает пока есть питание на MK
    rtt_init_print!();
    rprintln!("Вариант 6 (PAC): старт");

    let dp = stm32f411::Peripherals::take().unwrap(); // Забираем доступ ко всей периферии

    // Тактирование портов A, B, C
    dp.RCC
        .ahb1enr()
        .modify(|_, w| w.gpioaen().set_bit().gpioben().set_bit().gpiocen().set_bit());

    // Ставим режим пинов для светодиодов: выход
    dp.GPIOA
        .moder()
        .modify(|_, w| unsafe { w.moder6().bits(0b01).moder9().bits(0b01) }); // PA6 и PA9 выход
    dp.GPIOB
        .moder()
        .modify(|_, w| unsafe { w.moder6().bits(0b01) }); // PB6 выход

    // Славим подтяжку кнопок через pull-up
    dp.GPIOB
        .pupdr()
        .modify(|_, w| unsafe { w.pupdr3().bits(0b01) }); // PB3 в режиме pull-up (подтяжка к +3.3В)
    dp.GPIOC
        .pupdr()
        .modify(|_, w| unsafe { w.pupdr13().bits(0b01) }); // PC13 в режиме pull-up (подтяжка к +3.3В)

    let mut lit_count: u8 = 0; // Счётчик горящих диодов
    let mut turn_on_mode = true; // Флаг режима первой кнопки
    let mut prev_btn1 = false; // Флаг нажатия на первую кнопку
    let mut prev_btn2 = false; // Флаг нажатия на вторую кнопку

    apply_leds(&dp, lit_count);
    rprintln!("Режим D3: включение");

    loop {
        let raw1 = dp.GPIOB.idr().read().idr3().bit_is_clear(); // PB3: 0 = нажата (pull-up)
        let raw2 = dp.GPIOC.idr().read().idr13().bit_is_clear(); // PC13: 0 = нажата (pull-up)

        helper_delay_ms(5); // Защита от дребезга

        let confirm1 = dp.GPIOB.idr().read().idr3().bit_is_clear();
        let confirm2 = dp.GPIOC.idr().read().idr13().bit_is_clear();

        // BTN1 (D3): включает/выключает диоды
        if raw1 == confirm1 { // Проверяем, что значение не уплыло за время задержки
            if raw1 && !prev_btn1 { // Ловим момент нажатия
                if turn_on_mode {
                    if lit_count < 3 {
                        lit_count += 1;
                    }
                } else if lit_count > 0 {
                    lit_count -= 1;
                }
                apply_leds(&dp, lit_count);
                rprintln!("D3: lit_count = {}", lit_count);
            }
            prev_btn1 = raw1;
        }

        // BTN2 (B1): только переключает режим
        if raw2 == confirm2 {
            if raw2 && !prev_btn2 {
                turn_on_mode = !turn_on_mode;
                rprintln!(
                    "B1: режим D3 теперь {}",
                    if turn_on_mode { "включение" } else { "выключение" }
                );
            }
            prev_btn2 = raw2;
        }

        helper_delay_ms(5);
    }
}
