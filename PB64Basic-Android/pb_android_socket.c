/* Android Winsock compatibility shim for PB64Basic.
 *
 * Bionic already provides socket/connect/bind/listen/accept/send/recv/
 * sendto/recvfrom/setsockopt/htons/htonl/ntohl/ntohs/inet_addr/
 * gethostbyname/getservbyname with compatible ABI layouts.  Only the
 * Winsock-specific names are missing; map them onto POSIX here.
 */
#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <unistd.h>

typedef unsigned int PB_SOCKET;

/* ioctlsocket command used by the runtime: FIONREAD. */
#define PB_FIONREAD 0x4004667Fu

int closesocket(PB_SOCKET s) {
    return close((int)s);
}

int WSAStartup(unsigned short wVersionRequested, void* lpWSAData) {
    (void)wVersionRequested;
    if (lpWSAData) memset(lpWSAData, 0, 400); /* WSADATA is 400 bytes on x64. */
    return 0;
}

int WSACleanup(void) {
    return 0;
}

int WSAGetLastError(void) {
    if (errno == EAGAIN || errno == EWOULDBLOCK || errno == ETIMEDOUT)
        return 10060; /* WSAETIMEDOUT */
    if (errno == EINPROGRESS)
        return 10035; /* WSAEWOULDBLOCK */
    if (errno == ECONNRESET)
        return 10054; /* WSAECONNRESET */
    if (errno == ECONNREFUSED)
        return 10061; /* WSAECONNREFUSED */
    return errno;
}

int ioctlsocket(PB_SOCKET s, long cmd, unsigned long* argp) {
#ifdef FIONREAD
    if (cmd == PB_FIONREAD) {
        int avail = 0;
        if (!argp) {
            errno = EINVAL;
            return -1;
        }
        if (ioctl((int)s, FIONREAD, &avail) != 0) return -1;
        *argp = (unsigned long)avail;
        return 0;
    }
#endif
    (void)s;
    (void)argp;
    errno = EINVAL;
    return -1;
}

/* Android has no HWND message pump.  hwnd == 0 is the documented legacy
 * two-argument form (mask stored, no delivery), so accept it.  A real window
 * notification cannot be delivered and reports an error instead of silently
 * pretending to work. */
int WSAAsyncSelect(PB_SOCKET s, void* hWnd, unsigned int wMsg, long lEvent) {
    (void)s;
    (void)wMsg;
    (void)lEvent;
    if (!hWnd) return 0;
    errno = ENOTSUP;
    return -1;
}
