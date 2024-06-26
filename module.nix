{
  config,
  lib,
  pkgs,
  ...
}:
{
  options = with lib; {
    services.spambotsen = {
      enable = mkEnableOption ''
        Spambotsen discord bot
      '';

      package = mkOption {
        type = lib.types.package;
        default = pkgs.spambotsen;
      };

      configFile = mkOption { type = lib.types.path; };
    };
  };

  config = {
    systemd.services.spambotsen = lib.mkIf config.services.spambotsen.enable {
      wantedBy = [ "multi-user.target" ];
      after = [
        "network.target"
        "postgresql.service"
      ];
      wants = [ "network-online.target" ];
      restartIfChanged = true;

      serviceConfig = {
        user = "spambotsen";
        group = "spambotsen";
        restart = "always";
        WorkingDirectory =
          let
            static = pkgs.stdenv.mkDerivation {
              name = "spambotsen-static";
              src = ./.;
              phases = [ "installPhase" ];
              installPhase = ''
                mkdir -p $out
                cp -r $src/static $out
              '';
            };
          in
          static;
        ExecStart = "${config.services.spambotsen.package}/bin/spambotsen ${config.services.spambotsen.configFile}";
      };
    };
  };
}
