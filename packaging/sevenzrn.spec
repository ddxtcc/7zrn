Name:           sevenzrn
Version:        %{version}
Release:        1
Summary:        Native GPUI archive manager powered by 7-Zip and tar
License:        MIT
URL:            https://github.com/ddxtcc/7zrn
BuildArch:      x86_64
Requires:       tar
Requires:       (7zip or p7zip)

%description
7zrn is a dual-pane archive manager for Linux with archive creation,
extraction, encryption, previews, and desktop integration.

%prep

%build

%install
install -Dm755 %{bin} %{buildroot}%{_bindir}/sevenzrn
install -Dm644 %{repo}/packaging/7zrn.desktop %{buildroot}%{_datadir}/applications/7zrn.desktop
mkdir -p %{buildroot}%{_datadir}/icons
cp -a %{repo}/assets/icons/hicolor %{buildroot}%{_datadir}/icons/
install -Dm644 %{repo}/LICENSE %{buildroot}%{_datadir}/licenses/sevenzrn/LICENSE

%files
%{_bindir}/sevenzrn
%{_datadir}/applications/7zrn.desktop
%{_datadir}/icons/hicolor/*/apps/7zrn.*
%{_datadir}/licenses/sevenzrn/LICENSE
