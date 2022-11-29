# README

Контроллер для дисплея ГИП10000.
Аноды - 2xSN755870
Катоды - массив мультиплексоров SN74HC238D


# [cargo make](https://sagiegurari.github.io/cargo-make/)
1. flash - use openocd
2. log - defmt log, stagt debuginf first!

## Connections
Anodes: 
 * SO - PA7
 * SCK - PA5
 * LATCH - PA1

 ## Test environment 
 * Подавать на SN755870 VDDH не менее 27В, иначе на выходах высокий уровень не появляется вообще!
 