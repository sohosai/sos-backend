{ config, pkgs, ... }:
{
  security.acme = {
    acceptTerms = true;
    email = "info@sohosai.com";
  };
}
