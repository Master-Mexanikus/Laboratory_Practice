#![no_std]
#![no_main]

// Импорты и точка входа в программу
use stm32f4::stm32f411 as _;
use cortex_m as _;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

// Макросы - аналог #define SET_BIT/READ_BIT из С
macro_rules! set_bits { // Задаём новый макрос set_bits
    ($reg:expr, $mask:expr) => { // Шаблон для двух аргументов
        unsafe {
            let current = core::ptr::read_volatile($reg);
            core::ptr::write_volatile($reg, current | $mask); // Включаем нужные биты, остальные не трогаем
        }
    };
}

macro_rules! clear_bits { // Задаём новый макрос clear_bits
    ($reg:expr, $mask:expr) => {
        unsafe {
            let current = core::ptr::read_volatile($reg);
            core::ptr::write_volatile($reg, current & !$mask); // Выключаем нужные биты
        }
    };
}

macro_rules! bit_is_clear { // Задаём новый макрос bit_is_clear
    ($reg:expr, $mask:expr) => {
        unsafe { core::ptr::read_volatile($reg) & $mask == 0 } // true, если нужный бит равен нулю
    };
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

// Реализация управления светодиодами через макросы
fn helper_leds_manager(lit_count: u8) {
    if lit_count >= 1 {
        set_bits!(GPIOA_ODR, 1 << 6); // PA6 (красный)
    } else {
        clear_bits!(GPIOA_ODR, 1 << 6);
    }
    if lit_count >= 3 {
        set_bits!(GPIOA_ODR, 1 << 9); // PA9 (зелёный)
    } else {
        clear_bits!(GPIOA_ODR, 1 << 9);
    }
    if lit_count >= 2 {
        set_bits!(GPIOB_ODR, 1 << 6); // PB6 (жёлтый)
    } else {
        clear_bits!(GPIOB_ODR, 1 << 6);
    }
}

// Адреса регистров и управление тактированием (по RM0383, Reference Manual STM32F411RE)
const RCC_AHB1ENR: *mut u32 = 0x4002_3830 as *mut u32;

// Берём адреса А, В, С групп пинов МК, ориентируясь на наши пины подключения
const GPIOA_BASE: u32 = 0x4002_0000;
const GPIOB_BASE: u32 = 0x4002_0400;
const GPIOC_BASE: u32 = 0x4002_0800;

// Вычисляем (по RM0383) реальный адрес регистров для светодиодов. Смещение (0x00)
const GPIOA_MODER: *mut u32 = GPIOA_BASE as *mut u32;
const GPIOB_MODER: *mut u32 = GPIOB_BASE as *mut u32;
const GPIOC_MODER: *mut u32 = GPIOC_BASE as *mut u32;

// Вычисляем по смещению 0x0C (по RM0383) реальный адрес регистров для кнопок
const GPIOB_PUPDR: *mut u32 = (GPIOB_BASE + 0x0C) as *mut u32;
const GPIOC_PUPDR: *mut u32 = (GPIOC_BASE + 0x0C) as *mut u32;

// Адреса для чтения кнопок и управления диодами
const GPIOB_IDR: *mut u32 = (GPIOB_BASE + 0x10) as *mut u32;
const GPIOC_IDR: *mut u32 = (GPIOC_BASE + 0x10) as *mut u32;

const GPIOA_ODR: *mut u32 = (GPIOA_BASE + 0x14) as *mut u32;
const GPIOB_ODR: *mut u32 = (GPIOB_BASE + 0x14) as *mut u32;

// Точка входа
#[entry]
fn main() -> ! { // -> ! означает, что программа работает пока есть питание на MK
    rtt_init_print!();
    rprintln!("Вариант 6 (с макросами): старт");

    set_bits!(RCC_AHB1ENR, (1 << 0) | (1 << 1) | (1 << 2)); // Тактирование портов A, B, C

    set_bits!(GPIOA_MODER, (1 << 12) | (1 << 18)); // PA6, PA9 выход
    set_bits!(GPIOB_MODER, 1 << 12); // PB6  выход

    set_bits!(GPIOB_PUPDR, 1 << 6); // PB3 в режиме pull-up (подтяжка к +3.3В)
    set_bits!(GPIOC_PUPDR, 1 << 26); // PC13 в режиме pull-up (подтяжка к +3.3В)

    let mut lit_count: u8 = 0; // Счётчик горящих диодов
    let mut turn_on_mode = true; // Флаг режима первой кнопки
    let mut prev_btn1 = false; // Флаг нажатия на первую кнопку
    let mut prev_btn2 = false; // Флаг нажатия на вторую кнопку

    helper_leds_manager(lit_count);
    rprintln!("Режим D3: включение");

    loop {
        let raw1 = bit_is_clear!(GPIOB_IDR, 1 << 3); // PB3: 0 = нажата (pull-up)
        let raw2 = bit_is_clear!(GPIOC_IDR, 1 << 13); // PC13: 0 = нажата (pull-up)

        helper_delay_ms(5); // Защита от дребезга

        let confirm1 = bit_is_clear!(GPIOB_IDR, 1 << 3);
        let confirm2 = bit_is_clear!(GPIOC_IDR, 1 << 13);

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
                helper_leds_manager(lit_count);
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
