/*
C functions / values for implementing Fix standard library.
- When running program by `fix run`, then this source file will be compiled into shared library and loaded to the JIT environment.
- When running program by `fix build`, then this source file will be compiled into object file and linked to the binary.
*/

#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#ifndef __MINGW32__
#include <sys/time.h>
#endif // __MINGW32__
#include <time.h>
#include <unistd.h>

#ifdef __MINGW32__
#define timegm _mkgmtime

struct tm *localtime_r(const time_t *timer, struct tm *buf)
{
    if (localtime_s(buf, timer))
    {
        return NULL;
    }
    return buf;
}

struct tm *gmtime_r(const time_t *timer, struct tm *buf)
{
    if (gmtime_s(buf, timer))
    {
        return NULL;
    }
    return buf;
}
#endif // __MINGW32__

// Print message to stderr, and flush it.
void fixruntime_eprint(const char *msg)
{
    fprintf(stderr, "%s", msg);
    fflush(stderr);
}

void fixruntime_u8_to_bytes(uint8_t *buf, uint8_t v)
{
    *buf = v;
}
void fixruntime_u16_to_bytes(uint16_t *buf, uint16_t v)
{
    *buf = v;
}
void fixruntime_u32_to_bytes(uint32_t *buf, uint32_t v)
{
    *buf = v;
}
void fixruntime_u64_to_bytes(uint64_t *buf, uint64_t v)
{
    *buf = v;
}
void fixruntime_f32_to_bytes(float *buf, float v)
{
    *buf = v;
}
void fixruntime_f64_to_bytes(double *buf, double v)
{
    *buf = v;
}

uint8_t fixruntime_u8_from_bytes(uint8_t *buf)
{
    return *buf;
}
uint16_t fixruntime_u16_from_bytes(uint16_t *buf)
{
    return *buf;
}
uint32_t fixruntime_u32_from_bytes(uint32_t *buf)
{
    return *buf;
}
uint64_t fixruntime_u64_from_bytes(uint64_t *buf)
{
    return *buf;
}
float fixruntime_f32_from_bytes(float *buf)
{
    return *buf;
}
double fixruntime_f64_from_bytes(double *buf)
{
    return *buf;
}

void fixruntime_i8_to_str(char *buf, int8_t v)
{
    sprintf(buf, "%" PRId8, v);
}
void fixruntime_u8_to_str(char *buf, uint8_t v)
{
    sprintf(buf, "%" PRIu8, v);
}
void fixruntime_i16_to_str(char *buf, int16_t v)
{
    sprintf(buf, "%" PRId16, v);
}
void fixruntime_u16_to_str(char *buf, uint16_t v)
{
    sprintf(buf, "%" PRIu16, v);
}
void fixruntime_u32_to_str(char *buf, uint32_t v)
{
    sprintf(buf, "%" PRIu32, v);
}

void fixruntime_u64_to_str(char *buf, uint64_t v)
{
    sprintf(buf, "%" PRIu64, v);
}

void fixruntime_i32_to_str(char *buf, int32_t v)
{
    sprintf(buf, "%" PRId32, v);
}

void fixruntime_i64_to_str(char *buf, int64_t v)
{
    sprintf(buf, "%" PRId64, v);
}

void fixruntime_f32_to_str(char *buf, float v)
{
    sprintf(buf, "%f", v);
}

void fixruntime_f32_to_str_exp(char *buf, float v)
{
    sprintf(buf, "%e", v);
}

void fixruntime_f32_to_str_exp_precision(char *buf, float v, uint8_t precision)
{
    char specifier[7]; // len(%.255e) + 1
    sprintf(specifier, "%%.%" PRIu8 "e", precision);
    sprintf(buf, specifier, v);
}

void fixruntime_f32_to_str_precision(char *buf, float v, uint8_t precision)
{
    char specifier[7]; // len(%.255f) + 1
    sprintf(specifier, "%%.%" PRIu8 "f", precision);
    sprintf(buf, specifier, v);
}

void fixruntime_f64_to_str(char *buf, double v)
{
    sprintf(buf, "%lf", v);
}

void fixruntime_f64_to_str_exp(char *buf, double v)
{
    sprintf(buf, "%le", v);
}

void fixruntime_f64_to_str_exp_precision(char *buf, double v, uint8_t precision)
{
    char specifier[8]; // len(%.255le) + 1
    sprintf(specifier, "%%.%" PRIu8 "le", precision);
    sprintf(buf, specifier, v);
}

void fixruntime_f64_to_str_precision(char *buf, double v, uint8_t precision)
{
    char specifier[8]; // len(%.255lf) + 1
    sprintf(specifier, "%%.%" PRIu8 "lf", precision);
    sprintf(buf, specifier, v);
}

int64_t fixruntime_strtoll_10(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    int64_t v = (int64_t)strtoll(str, &endptr, 10);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

uint64_t fixruntime_strtoull_10(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    uint64_t v = (uint64_t)strtoull(str, &endptr, 10);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

double fixruntime_strtod(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    double v = strtod(str, &endptr);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

float fixruntime_strtof(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    float v = strtof(str, &endptr);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

void fixruntime_clock_gettime(int64_t *ret)
{
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ret[0] = (int64_t)ts.tv_sec;
    ret[1] = (int64_t)ts.tv_nsec;
}
void fixruntime_gmlocaltime(uint8_t is_local, uint64_t sec, int64_t *ret)
{
    // struct tm *gmtime_r(const time_t *timep, struct tm *result);
    time_t time = (time_t)sec;
    struct tm datetime;
    struct tm *is_suc;
    if (is_local > 0)
    {
        is_suc = localtime_r(&time, &datetime);
    }
    else
    {
        is_suc = gmtime_r(&time, &datetime);
    }
    ret[0] = (int64_t)datetime.tm_sec;
    ret[1] = (int64_t)datetime.tm_min;
    ret[2] = (int64_t)datetime.tm_hour;
    ret[3] = (int64_t)datetime.tm_mday;
    ret[4] = (int64_t)datetime.tm_mon;
    ret[5] = (int64_t)datetime.tm_year;
    ret[6] = (int64_t)datetime.tm_wday;
    ret[7] = (int64_t)datetime.tm_yday;
    ret[8] = (int64_t)datetime.tm_isdst;
    ret[9] = (int64_t)(is_suc == NULL);
}
int64_t fixruntime_timegmlocal(uint8_t is_local, int64_t *data)
{
    struct tm datetime;
    datetime.tm_sec = (int)data[0];
    datetime.tm_min = (int)data[1];
    datetime.tm_hour = (int)data[2];
    datetime.tm_mday = (int)data[3];
    datetime.tm_mon = (int)data[4];
    datetime.tm_year = (int)data[5];
    datetime.tm_wday = (int)data[6];
    datetime.tm_yday = (int)data[7];
    datetime.tm_isdst = (int)data[8];
    time_t ret;
    if (is_local > 0)
    {
        ret = mktime(&datetime);
    }
    else
    {
        ret = timegm(&datetime);
    }
    return (int64_t)ret;
}

int64_t fixruntime_clock()
{
    return (int64_t)clock();
}

double fixruntime_clocks_to_sec(int64_t clocks)
{
    return (double)(clock_t)clocks / CLOCKS_PER_SEC;
}

uint8_t fixruntime_is_einval()
{
    return errno == EINVAL;
}

uint8_t fixruntime_is_erange()
{
    return errno == ERANGE;
}

// Fork child process and launch process by execvp.
// * `error_buf` - If no error occurrs, error_buf will be set to pointing NULL.
//                 Otherwise, error_buf will be set to pointing to null-terminated error string. In this case, the caller should free the string buffer.
// * `streams` - If succceeds, streams[0], streams[1] and streams[2] are set FILE handles that are piped to stdio, stdout and stderr of child process.
void fixruntime_fork_execvp(const char *program_path, const char *argv[], char **out_error, FILE *out_streams[], int64_t *out_pid)
{
    *out_error = NULL;

    int pipes[3][2]; // in, out, err

    for (int i = 0; i < 3; i++)
    {
        if (pipe(pipes[i]))
        {
            // Failed creating pipes.
            for (int j = 0; j < i; j++)
            {
                close(pipes[j][0]);
                close(pipes[j][1]);
            }

            const char *msg = "Failed to create pipe.";
            *out_error = (char *)malloc(sizeof(char) * (strlen(msg) + 1));
            strcpy(*out_error, msg);

            return;
        }
    }
    pid_t pid = fork();
    if (!pid)
    {
        // In child process,

        dup2(pipes[0][0], 0); // stdin
        dup2(pipes[1][1], 1); // stdout
        dup2(pipes[2][1], 2); // stderr

        for (int i = 0; i < 3; i++)
        {
            close(pipes[i][0]);
            close(pipes[i][1]);
        }

        execvp(program_path, argv);

        // If execvp fails,
        exit(1);
    }
    else
    { // In parent process,
        if (pid < 0)
        {
            // Failed creating process.

            const char *msg = "Failed to create child process.";
            *out_error = (char *)malloc(sizeof(char) * (strlen(msg) + 1));
            strcpy(*out_error, msg);

            return;
        }
        close(pipes[0][0]);
        close(pipes[1][1]);
        close(pipes[2][1]);

        out_streams[0] = pipes[0][1];
        out_streams[1] = pipes[1][0];
        out_streams[2] = pipes[2][0];

        *out_pid = (int64_t)pid;

        return;
    }
}

// Wait termination of child process specified.
// * `timeout` - NULL or pointing to the timeout value (in seconds).
// * `out_is_timeout` - Set to 1 when return by timeout, or set to 0 otherwise. Should not be NULL when `timeout` is not NULL.
// * `out_wait_failed` - Set to 1 when waiting child process failed, or set to 0 otherwise.
// * `out_exit_status` - The exit status of child process is stored to the address specified this argument. This value should be used only when `*out_exit_status_available == 1`.
// * `out_exit_status_available` - Set to 1 when exit status is available, or set to 0 otherwise.
// * `out_stop_signal` - The signal number which caused the termination of the child process. This value should be used only when `*out_stop_signal_available == 1`.
// * `out_stop_signal_available` - Set to 1 when the stop signal number is available, or set to 0 otherwise.
void fixruntime_wait_subprocess(int64_t pid, double *timeout, uint8_t *out_is_timeout, uint8_t *out_wait_failed, uint8_t *out_exit_status, uint8_t *out_exit_status_available, uint8_t *out_stop_signal, uint8_t *out_stop_signal_available)
{
    int wait_status;
    pid_t wait_return;
    struct timespec start;
    double start_f;
    struct timespec now;
    double now_f;

    *out_is_timeout = 0;
    *out_exit_status_available = 0;
    *out_stop_signal_available = 0;
    *out_wait_failed = 0;

    if (timeout == NULL)
    {
        wait_return = waitpid((pid_t)pid, &wait_status, 0);
    }
    else
    {
        clock_gettime(CLOCK_MONOTONIC, &start);
        start_f = (double)start.tv_sec + (double)start.tv_nsec / 1e9;
        while (1)
        {
            // TODO: fix busy wait (using threads?)
            wait_return = waitpid((pid_t)pid, &wait_status, WNOHANG);
            if (wait_return != 0)
            {
                break;
            }
            clock_gettime(CLOCK_MONOTONIC, &now);
            now_f = (double)now.tv_sec + (double)now.tv_nsec / 1e9;
            if (now_f - start_f >= *timeout)
            {
                *out_is_timeout = 1;
                break;
            }
        }
    }
    if (wait_return == -1)
    {
        *out_wait_failed = 1;
        return;
    }
    *out_exit_status_available = (uint8_t)(WIFEXITED(wait_status) != 0);
    if (*out_exit_status_available)
    {
        *out_exit_status = (uint8_t)WEXITSTATUS(wait_status);
    }

    *out_stop_signal_available = (uint8_t)(WIFSIGNALED(wait_status) != 0);
    if (*out_stop_signal_available)
    {
        *out_stop_signal = (uint8_t)WSTOPSIG(wait_status);
    }
}