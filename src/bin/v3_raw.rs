// Настройки компилятора для сборки кода без ОС
#![no_std] // Отказ от стандартной библиотеки, тк она не поддерживается без ОС
#![no_main] // Отказ от станартной точки входа, чтобы компилятор не искал функцию main.

// Импорты и точка входа в программу
use stm32f4::stm32f411 as _; // Необходим, чтобы подтянуть таблицу векторов прерываний для cortex_m_rt
use cortex_m as _; // Для no_std Rust всегда требует использования cortex_m
use cortex_m_rt::entry; // Процедурный атребут для точки входа
use panic_rtt_target as _; // Для no_std Rust всегда требует инструкций на случай ошибок.
use rtt_target::{rprintln, rtt_init_print}; // Подключаем для отладочного вывода

// Реализация задержки
fn helper_delay_ms(ms: u32) { // Читаем временную переменную, чтобы создать задержку
    for _ in 0..ms {
        for _ in 0..4000 { // 4000 итераций 1 мс при HSI в 16 МГц
            unsafe { // Через ключевое слово unsafe отключаем проверку безопасности компилятором
                core::ptr::read_volatile(&0u32 as *const u32); // Создали пустую переменную и читаем её, явно прописав инструкцию компилятору
            }
        }
    }
}

// Реализация управления светодиодами
fn helper_leds_manager(lit_count: u8) { // Принимает число светодиодов, которые должны гореть
    unsafe {
        let mut odr_a = core::ptr::read_volatile(GPIOA_ODR); // Читаем текущее состояние регистра и сохраняем в ord_a
        if lit_count >= 1 {
            odr_a |= 1 << 6; // Включить PA6 (красный)
        } else {
            odr_a &= !(1 << 6); // Выключить PA6 (красный)
        }
        if lit_count >= 3 {
            odr_a |= 1 << 9; // Включить PA9 (зелёный)
        } else {
            odr_a &= !(1 << 9); // Выключить PA9 (зелёный)
        }

        core::ptr::write_volatile(GPIOA_ODR, odr_a); // Записываем в регистр значение из odr_a

        let mut odr_b = core::ptr::read_volatile(GPIOB_ODR); // Читаем текущее состояние регистра и сохраняем в ord_b
        if lit_count >= 2 {
            odr_b |= 1 << 6; // Включить PB6 (жёлтый)
        } else {
            odr_b &= !(1 << 6); // Выключить PB6 (жёлтый)
        }

        core::ptr::write_volatile(GPIOB_ODR, odr_b); // Записываем в регистр значение из odr_b
    }
}

// Адреса регистров и управление тактированием (по RM0383, Reference Manual STM32F411RE)
const RCC_AHB1ENR: *mut u32 = 0x4002_3830 as *mut u32; // Запоминаем адрес регистра, отвечющего за тактирование периферии

// Берём адреса А, В, С групп пинов МК, ориентируясь на наши пины поключения и запо
const GPIOA_BASE: u32 = 0x4002_0000; // Для PA6, PA9 запоминаем базовый адрес
const GPIOB_BASE: u32 = 0x4002_0400; // Для PB6, PB3 запоминаем базовый адрес
const GPIOC_BASE: u32 = 0x4002_0800; // Для PC13 запоминаем базовый адрес

// Вычмсляем (по RM0383) реальный адрес регистов для светодиодов. Смешение (0x00)
const GPIOA_MODER: *mut u32 = GPIOA_BASE as *mut u32; // Для пинов порта А
const GPIOB_MODER: *mut u32 = GPIOB_BASE as *mut u32; // Для пинов порта В
const GPIOC_MODER: *mut u32 = GPIOC_BASE as *mut u32; // Для пинов порта С

// Вычмсляем (по RM0383) реальный адрес регистов для подтяжки кнопок. Смешение (0x0С)
const GPIOB_PUPDR: *mut u32 = (GPIOB_BASE + 0x0C) as *mut u32; // Для подтяжки порта B (PB3)
const GPIOC_PUPDR: *mut u32 = (GPIOC_BASE + 0x0C) as *mut u32; // Для подтяжки порта C (PC13)

// Вычмсляем (по RM0383) реальный адрес регистов для чтения состояния кнопок. Смешение (0x10)
const GPIOB_IDR: *mut u32 = (GPIOB_BASE + 0x10) as *mut u32; // Чтение состояния PB3 (кнопка 1)
const GPIOC_IDR: *mut u32 = (GPIOC_BASE + 0x10) as *mut u32; // Чтение состояния PC13 (кнопка 2)

// Вычмсляем (по RM0383) реальный адрес регистов для записи стояния пинов. Смешение (0x14)
const GPIOA_ODR: *mut u32 = (GPIOA_BASE + 0x14) as *mut u32; // Управление PA6 и PA9
const GPIOB_ODR: *mut u32 = (GPIOB_BASE + 0x14) as *mut u32; // Управление PB6


// Точка входа
#[entry]
fn main() -> ! { // -> ! означает, что программа работает пока есть питание на MK
    rtt_init_print!();
    rprintln!("Вариант 6: три диода и две кнопки");

    // Изменяем RCC_AHB1ENR (включаем тактирование на группы пинов А, В, С)
    unsafe {
        let current = core::ptr::read_volatile(RCC_AHB1ENR); // Читаем RCC_AHB1ENR
        core::ptr::write_volatile(RCC_AHB1ENR, current | (1 << 0) | (1 << 1) | (1 << 2)); // Изменяем RCC_AHB1ENR
    }

    // Настраиваем режим пинов светодиодов
    unsafe {
        let moder_a = core::ptr::read_volatile(GPIOA_MODER);
        core::ptr::write_volatile(GPIOA_MODER, moder_a | (1 << 12) | (1 << 18)); // Выход для PA6 и PA9

        let moder_b = core::ptr::read_volatile(GPIOB_MODER);
        core::ptr::write_volatile(GPIOB_MODER, moder_b | (1 << 12)); // Выход для PB6
    }

    // Наитраиваем режим пинов кнопок
    unsafe {
        let pupdr_b = core::ptr::read_volatile(GPIOB_PUPDR);
        core::ptr::write_volatile(GPIOB_PUPDR, pupdr_b | (1 << 6)); // PB3 в режите pull-up (подтяжка к +3.3В)

        let pupdr_c = core::ptr::read_volatile(GPIOC_PUPDR);
        core::ptr::write_volatile(GPIOC_PUPDR, pupdr_c | (1 << 26)); // PC13 в режиме pull-up (подтяжка к +3.3В)
    }

    let mut lit_count: u8 = 0; // Счетчик горящих диодов
    let mut turn_on_mode = true; // Флаг режима первой кнопки
    let mut prev_btn1 = false; // Флаг нажатия на первую кнопку
    let mut prev_btn2 = false; // Флаг нажатия на второую кнопку

    loop {
        let idr_b = unsafe { core::ptr::read_volatile(GPIOB_IDR) }; // Читаем состояние порта В 
        let idr_c = unsafe { core::ptr::read_volatile(GPIOC_IDR) }; // Читаем состояние порта С
        let raw1 = idr_b & (1 << 3) == 0; // Читаем состояние PB3
        let raw2 = idr_c & (1 << 13) == 0; // Читаем состояние PC13

        helper_delay_ms(5); // Защита от дребезга

        let idr_b2 = unsafe { core::ptr::read_volatile(GPIOB_IDR) };
        let idr_c2 = unsafe { core::ptr::read_volatile(GPIOC_IDR) };
        let confirm1 = idr_b2 & (1 << 3) == 0;
        let confirm2 = idr_c2 & (1 << 13) == 0;

        // BTN1 (D3): включает/выключает диоды
        if raw1 == confirm1 { // Проверка совпадение данных
            if raw1 && !prev_btn1 { // Нажата в первый раз 
                if turn_on_mode { // Если режим по умолчанию стоит (включение), то работаем с добавлением числа включаемых диодов
                    if lit_count < 3 {
                        lit_count += 1;
                    }
                } else if lit_count > 0 { // Если режим выключения, то работаем с выключением
                    lit_count -= 1;
                }
                helper_leds_manager(lit_count); // Передаём параметры в управление диодами
                rprintln!("D3: lit_count = {}", lit_count);
            }
            prev_btn1 = raw1; // Запоминаем новое состояние
        }

        // BTN2 (B1): только переключает режим
        if raw2 == confirm2 { // Проверка совпадения данных
            if raw2 && !prev_btn2 { // Если нажата в первый раз
                turn_on_mode = !turn_on_mode; // Смена режима
                rprintln!(
                    "B1: режим D3 теперь {}",
                    if turn_on_mode { "включение" } else { "выключение" }
                );
            }
            prev_btn2 = raw2; // Запоминаем новое состояние
        }

        helper_delay_ms(5);
    }
}
