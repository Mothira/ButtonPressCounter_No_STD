#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_println::println;
use esp_hal::
{
    delay::Delay,
    prelude::*, 
    gpio::{Event, Input, Pull, Io, Level, Output}
};

// Create a Global Variable for a GPIO Peripheral to pass around between threads
static G_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
// Create a GLobal Variable for a FLAG to pass around between threads
// Option is not used since the variable is being directly initialized
// Using Cell instead of RefCell because it's a bool so we can take a copy instead of a reference
static G_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

// Global variable for the overall button count
static COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

// ISR Definition
#[handler]
fn gpio()
{
    // Start critical section
    critical_section::with(|cs| 
    {  
        // Obtain access to Global Input Peripheral
        // Clear Interrupt Pending Flag
        G_PIN.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();

        // Assert G_FLAG indicating a press button happened
        G_FLAG.borrow(cs).set(true);
    });

    critical_section::with(|cs| {
            if G_FLAG.borrow(cs).get()
            {
                // Clear global flag
                G_FLAG.borrow(cs).set(false);
                // Increment count and print to console
                critical_section::with(|cs| { *COUNT.borrow_ref_mut(cs) += 1;
                    println!("Button press count: {}", COUNT.borrow_ref_mut(cs));
                });
            }
        });
}

#[entry]
fn main() -> ! 
{
    // Create delay handler
    let delay = Delay::new();

    // Take Peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Create IO Driver
    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Interrupt Configuration
    io.set_interrupt_handler(gpio);
    let mut some_pin = Input::new(io.pins.gpio0, Pull::Up);
    some_pin.listen(Event::FallingEdge);
    critical_section::with(|cs| {  G_PIN.borrow_ref_mut(cs).replace(some_pin) });

    loop 
    {
        
    }
}
