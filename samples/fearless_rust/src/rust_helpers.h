#pragma once

#include <zephyr/net/net_mgmt.h>

void rust_panic(void);
int rust_errno(void);

struct rust_net_mgmt_cb {
    struct net_mgmt_event_callback net_mgmt_cb;
    void* rust_data;
};

void rust_net_mgmt_add_event_callback(struct rust_net_mgmt_cb* rust_cb, uint32_t mgmt_event_mask, void* rust_data);
void rust_net_mgmt_del_event_callback(struct rust_net_mgmt_cb* rust_cb);
