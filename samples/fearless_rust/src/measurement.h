#pragma once

#include <stdint.h>

struct measurement {
    float value;
    uint64_t device_id;
};

int measurement_get(struct measurement *output);
