#include "serializer.h"
#include <zephyr/kernel.h>
#include <zephyr/data/json.h>
#include <zephyr/logging/log.h>

LOG_MODULE_REGISTER(serializer);

struct measurement_json {
    const char* device_id;
    struct json_obj_token temperature;
};

static const struct json_obj_descr measurement_descr[] = {
    JSON_OBJ_DESCR_PRIM(struct measurement_json, device_id, JSON_TOK_STRING),
    JSON_OBJ_DESCR_PRIM(struct measurement_json, temperature, JSON_TOK_FLOAT),
};

int serializer_encode_measurement(const struct measurement *measurement, char *buffer, size_t buffer_size)
{
    char id_buf[32] = {0};
    sprintf(id_buf, "%llx", measurement->device_id);

    char temp_buf[32] = {0};
    int temp_len = sprintf(temp_buf, "%.3f", (double) measurement->temperature);

    struct measurement_json measurement_json = {
        .device_id = id_buf,
        .temperature = { .start = temp_buf, .length = temp_len },
    };

    int ret = json_obj_encode_buf(
        measurement_descr,
        ARRAY_SIZE(measurement_descr),
        &measurement_json,
        buffer,
        buffer_size
    );

    if (ret != 0) {
        LOG_ERR("Serialization failed: %d", ret);
        return ret;
    }

    return 0;
}
