%define name below
%define version 0.10.0
%define release 0%{?dist}
%define debug_package %{nil}
%global toolchain clang
%global _lto_cflags %{nil}

%if 0%{?rhel} == 9
%global clangver 16
%endif

%if 0%{?amzn} == 2023
%global clangver 18
%endif

Summary:        A time traveling resource monitor for modern Linux systems.
Name:           %{name}
Version:        %{version}
Release:        %{release}
License:        Apache-2.0 AND (MIT OR Apache-2.0) AND Unicode-DFS-2016 AND (Apache-2.0 OR BSL-1.0) AND (Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT) AND BSD-2-Clause AND (BSD-2-Clause OR Apache-2.0 OR MIT) AND (BSD-3-Clause OR MIT OR Apache-2.0) AND (LGPL-2.1-only OR BSD-2-Clause) AND MIT AND (MPL-2.0 OR MIT OR Apache-2.0) AND (Unlicense OR MIT)
URL:            https://github.com/facebookincubator/below
Source0:        https://github.com/facebookincubator/below/archive/refs/tags/v%{version}.tar.gz
Source1:        https://raw.githubusercontent.com/facebookincubator/below/main/etc/below.service
Source2:        https://raw.githubusercontent.com/facebookincubator/below/main/etc/logrotate.conf

ExcludeArch:    %{arm32} %{ix86}

BuildRequires:  curl
BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  gzip
BuildRequires:  ncurses-devel
BuildRequires:  zlib-devel
BuildRequires:  pkg-config
BuildRequires:  m4
BuildRequires:  elfutils-libelf-devel
BuildRequires:  systemd-rpm-macros

%if 0%{?rhel} == 9
BuildRequires: clang16
%endif

%if 0%{?amzn} == 2023
BuildRequires: clang18
%endif

Recommends:     logrotate

%description
below is an interactive tool to view and record historical system data.

%prep
%setup -q

%build
export CLANG=clang-%{?clangver}
export CC=clang-%{?clangver}
export CXX=clang++-%{?clangver}
# Install Rust using curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
export PATH="$PATH:$HOME/.cargo/bin"
rustup component add rustfmt
cargo build --release
strip --strip-all target/release/%{name}

%install
install -d %{buildroot}%{_bindir}
install -d %{buildroot}%{_datadir}/%{name}
install -d %{buildroot}%{_datadir}/%{name}/docs/
install -m 644 etc/* %{buildroot}%{_datadir}/%{name}
install -m 644 docs/* %{buildroot}%{_datadir}/%{name}/docs/
install -Dpm 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -D -p -m0644 %{SOURCE1} %{buildroot}%{_unitdir}/%{name}.service
install -D -p -m0644 %{SOURCE2} \
  %{buildroot}%{_sysconfdir}/logrotate.d/%{name}.conf
install -d -m1777 %{buildroot}%{_localstatedir}/log/%{name}

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

# %check
# cargo test -- --exact --skip test::record_replay_integration

%files
%license LICENSE
%doc README.md
%dir %{_sysconfdir}/logrotate.d
%{_unitdir}/%{name}.service
%config(noreplace) %{_sysconfdir}/logrotate.d/%{name}.conf
%{_bindir}/%{name}
%{_datadir}/%{name}/*
%dir %{_localstatedir}/log/%{name}

%changelog
* Sun July 27 2025 - Danie de Jager - 0.10.0-1
* Fri May 16 2025 - Danie de Jager - 0.9.0-1
* Mon Feb 10 2025 - Danie de Jager - 0.8.1-6
* Tue Nov 5 2024 - Danie de Jager - 0.8.1-5
* Tue Jun 11 2024 Danie de Jager - 0.8.1-3
- Add service and logrotate configs.
* Wed Feb 28 2024 Danie de Jager - 0.7.1-1
- Initial RPM build
