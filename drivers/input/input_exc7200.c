/*
 * Copyright (c) 2020,2023 NXP
 * Copyright (c) 2020 Mark Olsson <mark@markolsson.se>
 * Copyright (c) 2020 Teslabs Engineering S.L.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#define DT_DRV_COMPAT eeti_exc7200

#include <zephyr/drivers/gpio.h>
#include <zephyr/drivers/i2c.h>
#include <zephyr/input/input.h>

#include <zephyr/logging/log.h>
LOG_MODULE_REGISTER(exc7200, CONFIG_INPUT_LOG_LEVEL);

#define REPORTID_MTOUCH      0x04

#define EXC7200_READ_REG                             0x09
#define EXC7200_MAX_RAW_X                            2048
#define EXC7200_MAX_RAW_Y                            2048

struct exc7200_config {
	struct i2c_dt_spec bus;
	uint16_t screen_width;
	uint16_t screen_height;
};

struct exc7200_data {
	const struct device *dev;
	struct k_work work;
	struct k_timer timer;
};

static int exc7200_process(const struct device *dev)
{
	const struct exc7200_config *config = dev->config;
	struct exc7200_data *data = dev->data;

	uint8_t buf[10];
	int res = i2c_burst_read_dt(&config->bus, EXC7200_READ_REG, buf, sizeof(buf));

	if (res != 0) {
		LOG_ERR("Read failed! %d", res);
		return res;
	}

	if (buf[0] != REPORTID_MTOUCH) {
		// ignore any other
		return 0;
	}

	bool pressed = (buf[1] & 0x01) != 0;
	uint8_t contact_id = (buf[1] & 0x7C) >> 2;
	int32_t x = ((buf[3] << 8) + buf[2]) >> 4;
	int32_t y = ((buf[5] << 8) + buf[4]) >> 4;

	if (contact_id != 0) {
		// ignore any other
		return 0;
	}

	x = (x * config->screen_width) / EXC7200_MAX_RAW_X;
	y = (y * config->screen_height) / EXC7200_MAX_RAW_Y;

	LOG_DBG("pressed=%u x=%u y=%u", (unsigned) pressed, (unsigned) x, (unsigned) y);

	if (pressed) {
		input_report_abs(dev, INPUT_ABS_X, x, false, K_FOREVER);
		input_report_abs(dev, INPUT_ABS_Y, y, false, K_FOREVER);
		input_report_key(dev, INPUT_BTN_TOUCH, 1, true, K_FOREVER);
	} else {
		input_report_key(dev, INPUT_BTN_TOUCH, 0, true, K_FOREVER);
	}

	return 0;
}

static void exc7200_work_handler(struct k_work *work)
{
	struct exc7200_data *data = CONTAINER_OF(work, struct exc7200_data, work);

	exc7200_process(data->dev);
}

static void exc7200_timer_handler(struct k_timer *timer)
{
	struct exc7200_data *data = CONTAINER_OF(timer, struct exc7200_data, timer);

	k_work_submit(&data->work);
}

static int exc7200_init(const struct device *dev)
{
	const struct exc7200_config *config = dev->config;
	struct exc7200_data *data = dev->data;

	if (!device_is_ready(config->bus.bus)) {
		LOG_ERR("I2C controller device not ready");
		return -ENODEV;
	}

	data->dev = dev;

	k_work_init(&data->work, exc7200_work_handler);

	k_timer_init(&data->timer, exc7200_timer_handler, NULL);
	k_timer_start(&data->timer, K_MSEC(10), K_MSEC(10));

	return 0;
}

#define EXC7200_INIT(index)                                                         \
	static const struct exc7200_config exc7200_config_##index = {	                \
		.bus = I2C_DT_SPEC_INST_GET(index),                                         \
		.screen_width = DT_INST_PROP_OR(index, screen_width, EXC7200_MAX_RAW_X),    \
		.screen_height = DT_INST_PROP_OR(index, screen_height, EXC7200_MAX_RAW_Y),  \
	};								                                                \
	static struct exc7200_data exc7200_data_##index;			                    \
	DEVICE_DT_INST_DEFINE(index, exc7200_init, NULL,			                    \
			    &exc7200_data_##index, &exc7200_config_##index,                     \
			    POST_KERNEL, CONFIG_INPUT_INIT_PRIORITY, NULL);

DT_INST_FOREACH_STATUS_OKAY(EXC7200_INIT)
