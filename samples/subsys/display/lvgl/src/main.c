/*
 * Copyright (c) 2018 Jan Van Winkel <jan.van_winkel@dxplore.eu>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/display.h>
#include <zephyr/drivers/gpio.h>
#include <lvgl.h>
#include <stdio.h>
#include <string.h>
#include <zephyr/kernel.h>
#include <lvgl_input_device.h>

#define LOG_LEVEL CONFIG_LOG_DEFAULT_LEVEL

#include <zephyr/logging/log.h>

LOG_MODULE_REGISTER(app);

void lv_example_arc_1(void);
void lv_example_dropdown_2(void);
void lv_example_slider_1(void);
void lv_example_table_1(void);

int main(void) {
	const struct device *display_dev = DEVICE_DT_GET(DT_CHOSEN(zephyr_display));

	if (!device_is_ready(display_dev)) {
		LOG_ERR("Device not ready, aborting test");
		return 0;
	}

	//lv_example_arc_1();
	lv_example_dropdown_2();
	lv_example_slider_1();
	lv_example_table_1();

	lv_task_handler();
	display_blanking_off(display_dev);

	while (1) {
		lv_task_handler();
		k_sleep(K_MSEC(10));
	}
}

static void value_changed_event_cb(lv_event_t * e);

void lv_example_arc_1(void)
{
	lv_obj_t * label = lv_label_create(lv_scr_act());

	/*Create an Arc*/
	lv_obj_t * arc = lv_arc_create(lv_scr_act());
	lv_obj_set_size(arc, 150, 150);
	lv_arc_set_rotation(arc, 135);
	lv_arc_set_bg_angles(arc, 0, 270);
	lv_arc_set_value(arc, 10);
	lv_obj_align(arc, LV_ALIGN_CENTER, 0, -50);
	lv_obj_add_event_cb(arc, value_changed_event_cb, LV_EVENT_VALUE_CHANGED, label);

	/*Manually update the label for the first time*/
	lv_event_send(arc, LV_EVENT_VALUE_CHANGED, NULL);
}

/**
 * Create a drop down, up, left and right menus
 */
void lv_example_dropdown_2(void)
{
	static const char * opts = "Apple\n"
							   "Banana\n"
							   "Orange\n"
							   "Melon";

	lv_obj_t * dd;
	dd = lv_dropdown_create(lv_scr_act());
	lv_dropdown_set_options_static(dd, opts);
	lv_obj_align(dd, LV_ALIGN_TOP_MID, 0, 10);

	dd = lv_dropdown_create(lv_scr_act());
	lv_dropdown_set_options_static(dd, opts);
	lv_dropdown_set_dir(dd, LV_DIR_BOTTOM);
	lv_dropdown_set_symbol(dd, LV_SYMBOL_UP);
	lv_obj_align(dd, LV_ALIGN_BOTTOM_MID, 0, -10);

	dd = lv_dropdown_create(lv_scr_act());
	lv_dropdown_set_options_static(dd, opts);
	lv_dropdown_set_dir(dd, LV_DIR_RIGHT);
	lv_dropdown_set_symbol(dd, LV_SYMBOL_RIGHT);
	lv_obj_align(dd, LV_ALIGN_LEFT_MID, 10, 0);

	dd = lv_dropdown_create(lv_scr_act());
	lv_dropdown_set_options_static(dd, opts);
	lv_dropdown_set_dir(dd, LV_DIR_LEFT);
	lv_dropdown_set_symbol(dd, LV_SYMBOL_LEFT);
	lv_obj_align(dd, LV_ALIGN_RIGHT_MID, -10, 0);
}

static void value_changed_event_cb(lv_event_t * e)
{
	LOG_INF("callback");

	lv_obj_t * arc = lv_event_get_target(e);
	lv_obj_t * label = lv_event_get_user_data(e);

	lv_label_set_text_fmt(label, "%d%%", lv_arc_get_value(arc));

	/*Rotate the label to the current position of the arc*/
	lv_arc_rotate_obj_to_angle(arc, label, 25);
}

static void slider_event_cb(lv_event_t * e);
static lv_obj_t * slider_label;

/**
 * A default slider with a label displaying the current value
 */
void lv_example_slider_1(void)
{
	/*Create a slider in the center of the display*/
	lv_obj_t * slider = lv_slider_create(lv_scr_act());
	lv_obj_align(slider, LV_ALIGN_CENTER, 0, 100);
	lv_obj_add_event_cb(slider, slider_event_cb, LV_EVENT_VALUE_CHANGED, NULL);

	/*Create a label below the slider*/
	slider_label = lv_label_create(lv_scr_act());
	lv_label_set_text(slider_label, "0%");

	lv_obj_align_to(slider_label, slider, LV_ALIGN_OUT_BOTTOM_MID, 0, 10);
}

static void slider_event_cb(lv_event_t * e)
{
	lv_obj_t * slider = lv_event_get_target(e);
	char buf[8];
	lv_snprintf(buf, sizeof(buf), "%d%%", (int)lv_slider_get_value(slider));
	lv_label_set_text(slider_label, buf);
	lv_obj_align_to(slider_label, slider, LV_ALIGN_OUT_BOTTOM_MID, 0, 10);
}

static void draw_part_event_cb(lv_event_t * e)
{
	lv_obj_t * obj = lv_event_get_target(e);
	lv_obj_draw_part_dsc_t * dsc = lv_event_get_draw_part_dsc(e);
	/*If the cells are drawn...*/
	if(dsc->part == LV_PART_ITEMS) {
		uint32_t row = dsc->id /  lv_table_get_col_cnt(obj);
		uint32_t col = dsc->id - row * lv_table_get_col_cnt(obj);

		/*Make the texts in the first cell center aligned*/
		if(row == 0) {
			dsc->label_dsc->align = LV_TEXT_ALIGN_CENTER;
			dsc->rect_dsc->bg_color = lv_color_mix(lv_palette_main(LV_PALETTE_BLUE), dsc->rect_dsc->bg_color, LV_OPA_20);
			dsc->rect_dsc->bg_opa = LV_OPA_COVER;
		}
			/*In the first column align the texts to the right*/
		else if(col == 0) {
			dsc->label_dsc->align = LV_TEXT_ALIGN_RIGHT;
		}

		/*MAke every 2nd row grayish*/
		if((row != 0 && row % 2) == 0) {
			dsc->rect_dsc->bg_color = lv_color_mix(lv_palette_main(LV_PALETTE_GREY), dsc->rect_dsc->bg_color, LV_OPA_10);
			dsc->rect_dsc->bg_opa = LV_OPA_COVER;
		}
	}
}

void lv_example_table_1(void)
{
	lv_obj_t * table = lv_table_create(lv_scr_act());

	/*Fill the first column*/
	lv_table_set_cell_value(table, 0, 0, "Name");
	lv_table_set_cell_value(table, 1, 0, "Apple");
	lv_table_set_cell_value(table, 2, 0, "Banana");
	lv_table_set_cell_value(table, 3, 0, "Lemon");
	lv_table_set_cell_value(table, 4, 0, "Grape");
	lv_table_set_cell_value(table, 5, 0, "Melon");
	lv_table_set_cell_value(table, 6, 0, "Peach");
	lv_table_set_cell_value(table, 7, 0, "Nuts");

	/*Fill the second column*/
	lv_table_set_cell_value(table, 0, 1, "Price");
	lv_table_set_cell_value(table, 1, 1, "$7");
	lv_table_set_cell_value(table, 2, 1, "$4");
	lv_table_set_cell_value(table, 3, 1, "$6");
	lv_table_set_cell_value(table, 4, 1, "$2");
	lv_table_set_cell_value(table, 5, 1, "$5");
	lv_table_set_cell_value(table, 6, 1, "$1");
	lv_table_set_cell_value(table, 7, 1, "$9");

	/*Set a smaller height to the table. It'll make it scrollable*/
	lv_obj_set_height(table, 200);
	lv_obj_align(table, LV_ALIGN_CENTER, 0, -50);

	/*Add an event callback to to apply some custom drawing*/
	lv_obj_add_event_cb(table, draw_part_event_cb, LV_EVENT_DRAW_PART_BEGIN, NULL);
}
