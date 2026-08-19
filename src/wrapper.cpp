// osrm_wrapper.cpp
#include <osrm/osrm.hpp>
#include <osrm/table_parameters.hpp>
#include <osrm/engine_config.hpp>
#include <osrm/json_container.hpp>
#include <util/json_renderer.hpp>
#include <osrm/route_parameters.hpp>
#include <osrm/trip_parameters.hpp>

#include <string>
#include <iostream>
#include <cstdlib>
#include <cstring>

namespace {
    char* copy_message(const std::string& message) {
        char* copy = new char[message.size() + 1];
        std::memcpy(copy, message.c_str(), message.size() + 1);
        return copy;
    }

    std::string error_message(const osrm::json::Object& result) {
        try {
            return std::get<osrm::json::String>(result.values.at("message")).value;
        } catch (const std::exception&) {
            return "Unknown OSRM error";
        }
    }
}

extern "C" {

    struct OSRM_Result {
        int code;
        char* message;
    };

    struct OSRM_SimpleRouteResult {
        int code;
        double distance;
        double duration;
        char* message;
    };

    void* osrm_create(const char* base_path, const char* algorithm) {
        try {
            osrm::EngineConfig config;
            config.storage_config = {base_path};
            config.use_shared_memory = false;

            if (strcmp(algorithm, "CH") == 0) {
                config.algorithm = osrm::EngineConfig::Algorithm::CH;
            }
            else if (strcmp(algorithm, "MLD") == 0) {
                config.algorithm = osrm::EngineConfig::Algorithm::MLD;
            }
            else {
               config.algorithm = osrm::EngineConfig::Algorithm::MLD;
            }

            return new osrm::OSRM(config);
        } catch (const std::exception& e) {
            std::cerr << "Fail to create an OSRM instance: " << e.what() << std::endl;
            return nullptr;
        }
    }

    void osrm_destroy(void* osrm_instance) {
        if (osrm_instance) {
            delete static_cast<osrm::OSRM*>(osrm_instance);
        }
    }

    OSRM_Result osrm_table(void* osrm_instance,
                          const double* coordinates,
                          size_t num_coordinates,
                          size_t num_sources,
                          size_t num_destinations) {

        if (!osrm_instance) {
            return {1, copy_message("OSRM instance not found")};
        }

        osrm::OSRM* osrm_ptr = static_cast<osrm::OSRM*>(osrm_instance);
        osrm::TableParameters params;

        params.coordinates.reserve(num_coordinates);
        for (size_t i = 0; i < num_coordinates; ++i) {
            params.coordinates.push_back({
                osrm::util::FloatLongitude{coordinates[i * 2]},
                osrm::util::FloatLatitude{coordinates[i * 2 + 1]}
            });
        }

        params.sources.reserve(num_sources);
        for (size_t i = 0; i < num_sources; ++i) {
            params.sources.push_back(i);
        }

        params.destinations.reserve(num_destinations);
        for (size_t i = 0; i < num_destinations; ++i) {
            params.destinations.push_back(num_sources + i);
        }

        osrm::json::Object result;
        const auto status = osrm_ptr->Table(params, result);

        std::string result_str;
        int code;

        if (status == osrm::Status::Ok) {
            code = 0;
            osrm::util::json::render(result_str, result);
        } else {
            code = 1;
            result_str = error_message(result);
        }

        return {code, copy_message(result_str)};
    }

    OSRM_Result osrm_route(void* osrm_instance,
                           const double* coordinates,
                           size_t num_coordinates)
    {
        if (!osrm_instance) {
            return {1, copy_message("OSRM instance not found")};
        }

        osrm::OSRM* osrm_ptr = static_cast<osrm::OSRM*>(osrm_instance);
        osrm::RouteParameters params;

        params.coordinates.reserve(num_coordinates);
        for (size_t i = 0; i < num_coordinates; ++i) {
            params.coordinates.push_back({
                osrm::util::FloatLongitude{coordinates[i * 2]},
                osrm::util::FloatLatitude{coordinates[i * 2 + 1]}
            });
        }

        osrm::json::Object result;
        const auto status = osrm_ptr->Route(params, result);

        std::string result_str;
        int code;

        if (status == osrm::Status::Ok) {
            code = 0;
            osrm::util::json::render(result_str, result);
        } else {
            code = 1;
            result_str = error_message(result);
        }

        return {code, copy_message(result_str)};
    }

    OSRM_SimpleRouteResult osrm_simple_route(void* osrm_instance,
                                              double from_longitude,
                                              double from_latitude,
                                              double to_longitude,
                                              double to_latitude)
    {
        if (!osrm_instance) {
            return {1, 0.0, 0.0, copy_message("OSRM instance not found")};
        }

        try {
            auto* osrm_ptr = static_cast<osrm::OSRM*>(osrm_instance);
            osrm::RouteParameters params;
            params.coordinates.reserve(2);
            params.coordinates.push_back({
                osrm::util::FloatLongitude{from_longitude},
                osrm::util::FloatLatitude{from_latitude}
            });
            params.coordinates.push_back({
                osrm::util::FloatLongitude{to_longitude},
                osrm::util::FloatLatitude{to_latitude}
            });
            params.overview = osrm::RouteParameters::OverviewType::False;
            params.skip_waypoints = true;
            params.generate_hints = false;

            osrm::json::Object result;
            if (osrm_ptr->Route(params, result) != osrm::Status::Ok) {
                return {1, 0.0, 0.0, copy_message(error_message(result))};
            }

            const auto& routes =
                std::get<osrm::json::Array>(result.values.at("routes")).values;
            if (routes.empty()) {
                return {1, 0.0, 0.0, copy_message("No route was returned")};
            }

            const auto& route = std::get<osrm::json::Object>(routes.front());
            const auto distance =
                std::get<osrm::json::Number>(route.values.at("distance")).value;
            const auto duration =
                std::get<osrm::json::Number>(route.values.at("duration")).value;
            return {0, distance, duration, nullptr};
        } catch (const std::exception& exception) {
            return {1, 0.0, 0.0, copy_message(exception.what())};
        }
    }

    OSRM_Result osrm_trip(void* osrm_instance,
                          const double* coordinates,
                          size_t num_coordinates)
    {

            if (!osrm_instance) {
                return {1, copy_message("OSRM instance not found")};
            }

            osrm::OSRM* osrm_ptr = static_cast<osrm::OSRM*>(osrm_instance);
            osrm::TripParameters params;

            params.coordinates.reserve(num_coordinates);
            for (size_t i = 0; i < num_coordinates; ++i) {
                params.coordinates.push_back({
                    osrm::util::FloatLongitude{coordinates[i * 2]},
                    osrm::util::FloatLatitude{coordinates[i * 2 + 1]}
                });
            }

            osrm::json::Object result;
            const auto status = osrm_ptr->Trip(params, result);

            std::string result_str;
            int code;

            if (status == osrm::Status::Ok) {
                code = 0;
                osrm::util::json::render(result_str, result);
            } else {
                code = 1;
                result_str = error_message(result);
            }

            return {code, copy_message(result_str)};
        }

    void osrm_free_string(char* s) {
        if (s) {
            delete[] s;
        }
    }
}
