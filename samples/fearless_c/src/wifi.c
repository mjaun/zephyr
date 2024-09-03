#include "wifi.h"
#include <zephyr/init.h>
#include <zephyr/kernel.h>
#include <zephyr/net/net_if.h>
#include <zephyr/net/wifi_mgmt.h>
#include <zephyr/net/net_event.h>

// Include Git ignored file containing credentials. Must contain:
// #define WIFI_SSID     "<ssid>"
// #define WIFI_PSK      "<password>"
#include "wifi_credentials.h"

LOG_MODULE_REGISTER(wifi);

static void net_mgmt_event_handler(struct net_mgmt_event_callback *cb, uint32_t mgmt_event, struct net_if *iface);
static void handle_wifi_connect_result(struct net_mgmt_event_callback *cb, struct net_if *iface);
static void handle_wifi_disconnect_result(struct net_mgmt_event_callback *cb);
static void handle_ipv4_result(struct net_if *iface);

static struct net_mgmt_event_callback wifi_cb;
static struct net_mgmt_event_callback ipv4_cb;

static K_SEM_DEFINE(connected_sem, 0, 1);
static K_SEM_DEFINE(ip_obtained_sem, 0, 1);
static K_SEM_DEFINE(disconnected_sem, 0, 1);

void wifi_init()
{
    net_mgmt_init_event_callback(
        &wifi_cb,
        net_mgmt_event_handler,
        NET_EVENT_WIFI_CONNECT_RESULT | NET_EVENT_WIFI_DISCONNECT_RESULT
    );

    net_mgmt_init_event_callback(
        &ipv4_cb,
        net_mgmt_event_handler,
        NET_EVENT_IPV4_ADDR_ADD
    );

    net_mgmt_add_event_callback(&wifi_cb);
    net_mgmt_add_event_callback(&ipv4_cb);
}

int wifi_connect()
{
    struct net_if *iface = net_if_get_default();
    struct wifi_connect_req_params wifi_params = {};

    wifi_params.ssid = WIFI_SSID;
    wifi_params.psk = WIFI_PSK;
    wifi_params.ssid_length = strlen(WIFI_SSID);
    wifi_params.psk_length = strlen(WIFI_PSK);
    wifi_params.channel = WIFI_CHANNEL_ANY;
    wifi_params.security = WIFI_SECURITY_TYPE_PSK;
    wifi_params.band = WIFI_FREQ_BAND_2_4_GHZ;
    wifi_params.mfp = WIFI_MFP_OPTIONAL;

    LOG_INF("Connecting to SSID: %s", wifi_params.ssid);

    int ret = net_mgmt(NET_REQUEST_WIFI_CONNECT, iface, &wifi_params, sizeof(struct wifi_connect_req_params));

    if (ret != 0) {
        LOG_ERR("Connection request failed: %d", ret);
        return ret;
    }

    ret = k_sem_take(&connected_sem, K_SECONDS(15));

    if (ret != 0) {
        LOG_ERR("Failed to connect");
        return ret;
    }

    net_dhcpv4_start(iface);

    ret = k_sem_take(&ip_obtained_sem, K_SECONDS(15));

    if (ret != 0) {
        LOG_ERR("Failed to obtain IP address");
        return ret;
    }

    return 0;
}

int wifi_disconnect()
{
    struct net_if *iface = net_if_get_default();

    LOG_INF("Disconnecting");

    int ret = net_mgmt(NET_REQUEST_WIFI_DISCONNECT, iface, NULL, 0);

    if (ret != 0) {
        LOG_ERR("Disconnection request failed: %d", ret);
        return ret;
    }

    ret = k_sem_take(&disconnected_sem, K_SECONDS(5));

    if (ret != 0) {
        LOG_ERR("Failed to disconnect");
        return ret;
    }

    return 0;
}

static void net_mgmt_event_handler(struct net_mgmt_event_callback *cb, uint32_t mgmt_event, struct net_if *iface)
{
    switch (mgmt_event) {
        case NET_EVENT_WIFI_CONNECT_RESULT:
            handle_wifi_connect_result(cb, iface);
            break;

        case NET_EVENT_WIFI_DISCONNECT_RESULT:
            handle_wifi_disconnect_result(cb);
            break;

        case NET_EVENT_IPV4_ADDR_ADD:
            handle_ipv4_result(iface);
            break;

        default:
            break;
    }
}

static void handle_wifi_connect_result(struct net_mgmt_event_callback *cb, struct net_if *iface)
{
    const struct wifi_status *status = cb->info;

    if (status->status == 0) {
        LOG_INF("Connected");
        k_sem_give(&connected_sem);
    } else {
        LOG_ERR("Connection request failed: %d", status->status);
    }
}

static void handle_wifi_disconnect_result(struct net_mgmt_event_callback *cb)
{
    const struct wifi_status *status = cb->info;

    if (status->status == 0) {
        LOG_INF("Disconnected");
        k_sem_give(&disconnected_sem);
    } else {
        LOG_ERR("Disconnection request failed: %d", status->status);
    }
}

static void handle_ipv4_result(struct net_if *iface)
{
    for (int i = 0; i < NET_IF_MAX_IPV4_ADDR; i++) {
        char buf[NET_IPV4_ADDR_LEN];

        if (iface->config.ip.ipv4->unicast[i].ipv4.addr_type != NET_ADDR_DHCP) {
            continue;
        }

        LOG_INF("IP address obtained");
        LOG_INF("  Address: %s", net_addr_ntop(AF_INET, &iface->config.ip.ipv4->unicast[i].ipv4.address.in_addr, buf, sizeof(buf)));
        LOG_INF("  Netmask: %s", net_addr_ntop(AF_INET, &iface->config.ip.ipv4->unicast[i].netmask, buf, sizeof(buf)));
        LOG_INF("  Gateway: %s", net_addr_ntop(AF_INET, &iface->config.ip.ipv4->gw, buf, sizeof(buf)));
    }

    k_sem_give(&ip_obtained_sem);
}
