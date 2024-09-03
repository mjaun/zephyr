#include "measurement.h"
#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/sensor.h>
#include <zephyr/logging/log.h>
#include <zephyr/data/json.h>

LOG_MODULE_REGISTER(measurement);

#define SENSOR_NODE   DT_ALIAS(temp_sensor)

static const struct device *sensor_dev = DEVICE_DT_GET(SENSOR_NODE);

struct measurement_json {
    const char* device_id;
    struct json_obj_token temperature;
};

static const struct json_obj_descr measurement_descr[] = {
    JSON_OBJ_DESCR_PRIM(struct measurement_json, device_id, JSON_TOK_STRING),
    JSON_OBJ_DESCR_PRIM(struct measurement_json, temperature, JSON_TOK_FLOAT),
};

int measurement_get(struct measurement *output)
{
    struct sensor_value temp;
    int ret;

    if (!device_is_ready(sensor_dev)) {
        LOG_ERR("Sensor device not ready!");
        return -ENODEV;
    }

    ret = sensor_sample_fetch(sensor_dev);

    if (ret != 0) {
        LOG_ERR("Sample fetch failed: %d", ret);
        return ret;
    }

    ret = sensor_channel_get(sensor_dev, SENSOR_CHAN_AMBIENT_TEMP, &temp);

    if (ret != 0) {
        LOG_ERR("Channel get failed: %d", ret);
        return ret;
    }

    output->temperature = (float) temp.val1 + ((float) temp.val2 * 0.000001f);
    strcpy(output->device_id, "TODO");
    return 0;
}

int measurement_serialize(const struct measurement *measurement, char *buffer, size_t buffer_size)
{
    char float_buf[32] = {0};
    int float_len = snprintf(float_buf, sizeof(float_buf), "%.3f", (double) measurement->temperature);

    struct measurement_json measurement_json = {
        .device_id = measurement->device_id,
        .temperature = { .start = float_buf, .length = float_len },
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
