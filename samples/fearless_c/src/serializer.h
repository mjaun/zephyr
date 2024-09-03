#pragma once

#include <stdint.h>
#include <stdlib.h>

struct measurement {
    uint64_t device_id;
    float temperature;
};

int serializer_encode_measurement(const struct measurement* measurement, char* buffer, size_t buffer_size);
