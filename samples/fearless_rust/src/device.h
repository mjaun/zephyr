#pragma once

#include <stdint.h>

uint64_t device_id_get();

void device_schedule_wakeup(int seconds);
void device_deep_sleep();
