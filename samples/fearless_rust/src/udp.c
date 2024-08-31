#include "udp.h"
#include <zephyr/kernel.h>
#include <zephyr/net/socket.h>

// Include Git ignored file containing UDP server info. Must contain:
// #define UDP_SERVER    "<host>"
// #define UDP_PORT      <port>
#include "udp_server.h"

LOG_MODULE_REGISTER(udp);

static int udp_socket = 0;

int udp_init()
{
    struct sockaddr_in addr4;
    addr4.sin_family = AF_INET;
    addr4.sin_port = htons(UDP_PORT);

    zsock_inet_pton(AF_INET, UDP_SERVER, &addr4.sin_addr);

    udp_socket = zsock_socket(addr4.sin_family, SOCK_DGRAM, IPPROTO_UDP);

    if (udp_socket < 0) {
        LOG_ERR("Failed to create socket: %d", -errno);
        return -errno;
    }

    int ret = zsock_connect(udp_socket, (const struct sockaddr *) &addr4, sizeof(addr4));

    if (ret < 0) {
        LOG_ERR("Failed to connect UDP socket: %d", -errno);
        return -errno;
    }

    LOG_INF("UDP socket initialized");
    return 0;
}

int udp_send(const void *payload, size_t length)
{
    int ret = zsock_send(udp_socket, payload, length, 0);

    if (ret < 0) {
        LOG_ERR("Failed to send: %d", ret);
        return -EIO;
    }

    LOG_INF("Sent %u bytes", length);
    return 0;
}
