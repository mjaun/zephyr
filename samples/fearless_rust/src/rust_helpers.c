#include "rust_helpers.h"
#include <zephyr/kernel.h>
#include <zephyr/net/net_mgmt.h>

extern void rust_net_mgmt_event_handler(void* rust_data, struct net_if *iface, uint32_t mgmt_event, const void* info, size_t info_length);

static void net_mgmt_event_handler(struct net_mgmt_event_callback *cb, uint32_t mgmt_event, struct net_if *iface);

void rust_panic(void)
{
    k_panic();
}

int rust_errno(void)
{
    return errno;
}

void rust_net_mgmt_add_event_callback(struct rust_net_mgmt_cb *rust_cb, uint32_t mgmt_event_mask, void* rust_data)
{
    rust_cb->rust_data = rust_data;

    net_mgmt_init_event_callback(&rust_cb->net_mgmt_cb, net_mgmt_event_handler, mgmt_event_mask);
    net_mgmt_add_event_callback(&rust_cb->net_mgmt_cb);
}

void rust_net_mgmt_del_event_callback(struct rust_net_mgmt_cb *rust_cb)
{
    net_mgmt_del_event_callback(&rust_cb->net_mgmt_cb);
}

static void net_mgmt_event_handler(struct net_mgmt_event_callback *cb, uint32_t mgmt_event, struct net_if *iface)
{
    struct rust_net_mgmt_cb *rust_cb = CONTAINER_OF(cb, struct rust_net_mgmt_cb, net_mgmt_cb);
    rust_net_mgmt_event_handler(rust_cb->rust_data, iface, mgmt_event, cb->info, cb->info_length);
}
