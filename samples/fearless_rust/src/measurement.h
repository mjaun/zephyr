#pragma once

#include <stdint.h>
#include <stdlib.h>

struct measurement {
    char device_id[16];
    float temperature;
};

int measurement_get(struct measurement *output);
int measurement_serialize(const struct measurement* measurement, char* buffer, size_t buffer_size);
