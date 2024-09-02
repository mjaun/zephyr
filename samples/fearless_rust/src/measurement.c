#include "measurement.h"
#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/sensor.h>
#include <zephyr/logging/log.h>

LOG_MODULE_REGISTER(measurement);

#define SENSOR_NODE   DT_ALIAS(temp_sensor)

static const struct device *sensor_dev = DEVICE_DT_GET(SENSOR_NODE);

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

    output->value = (float) temp.val1 + ((float) temp.val2 * 0.000001f);
    output->device_id = 0;  // TODO
    return 0;
}
