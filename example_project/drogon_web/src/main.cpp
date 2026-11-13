// Exemplo com libs que dependem de pacotes -dev do sistema:
//   drogon (servidor HTTP), libpqxx (PostgreSQL), rtaudio (áudio)
//
// Rotas (http://127.0.0.1:8080):
//   /        -> texto de boas-vindas
//   /audio   -> lista os dispositivos de áudio (JSON)
//   /db      -> tenta conectar no PostgreSQL usando a variável DATABASE_URL
//               ex: DATABASE_URL="postgresql://user:senha@localhost/meubanco"

#include <cstdlib>
#include <functional>
#include <iostream>
#include <string>

#include <RtAudio.h>
#include <drogon/drogon.h>
#include <pqxx/pqxx>

using namespace drogon;

static Json::Value listar_audio() {
    Json::Value out(Json::objectValue);
    Json::Value devs(Json::arrayValue);
    try {
        RtAudio audio;
        out["api"] = RtAudio::getApiDisplayName(audio.getCurrentApi());
        for (unsigned int id : audio.getDeviceIds()) {
            RtAudio::DeviceInfo info = audio.getDeviceInfo(id);
            Json::Value d;
            d["id"] = id;
            d["nome"] = info.name;
            d["entradas"] = info.inputChannels;
            d["saidas"] = info.outputChannels;
            d["padrao_saida"] = info.isDefaultOutput;
            devs.append(d);
        }
    } catch (const std::exception& e) {
        out["erro"] = e.what();
    }
    out["dispositivos"] = devs;
    return out;
}

static Json::Value testar_db() {
    Json::Value out(Json::objectValue);
    const char* url = std::getenv("DATABASE_URL");
    if (!url) {
        out["erro"] = "defina DATABASE_URL (ex: postgresql://user:senha@localhost/meubanco)";
        return out;
    }
    try {
        pqxx::connection conn{url};
        pqxx::work tx{conn};
        auto row = tx.exec1("SELECT version()");
        out["versao"] = row[0].as<std::string>();
    } catch (const std::exception& e) {
        out["erro"] = e.what();
    }
    return out;
}

int main() {
    using Callback = std::function<void(const HttpResponsePtr&)>;

    app().registerHandler("/", [](const HttpRequestPtr&, Callback&& cb) {
        auto r = HttpResponse::newHttpResponse();
        r->setContentTypeCode(CT_TEXT_PLAIN);
        r->setBody("Ola do rsfmake! Rotas: /audio  /db\n");
        cb(r);
    });

    app().registerHandler("/audio", [](const HttpRequestPtr&, Callback&& cb) {
        cb(HttpResponse::newHttpJsonResponse(listar_audio()));
    });

    app().registerHandler("/db", [](const HttpRequestPtr&, Callback&& cb) {
        cb(HttpResponse::newHttpJsonResponse(testar_db()));
    });

    std::cout << "Servidor em http://127.0.0.1:8080  (Ctrl+C para sair)\n";
    app().addListener("127.0.0.1", 8080).run();
    return 0;
}
