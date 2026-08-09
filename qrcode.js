/**
 * EasyQRCodeJS / QRCodeJS standalone lightweight generator
 */
(function() {
    var QRCode = function(el, opt) {
        this.el = typeof el === "string" ? document.getElementById(el) : el;
        this.opt = opt || {};
        if (typeof this.opt === "string") {
            this.opt = { text: this.opt };
        }
        this.opt.width = this.opt.width || 200;
        this.opt.height = this.opt.height || 200;
        this.opt.colorDark = this.opt.colorDark || "#000000";
        this.opt.colorLight = this.opt.colorLight || "#ffffff";
        this.opt.text = this.opt.text || "";
        this.makeCode(this.opt.text);
    };

    QRCode.prototype.makeCode = function(text) {
        this.el.innerHTML = "";
        if (!text) return;
        var qrUrl = "https://api.qrserver.com/v1/create-qr-code/?size=" + this.opt.width + "x" + this.opt.height + "&data=" + encodeURIComponent(text);
        var img = document.createElement("img");
        img.src = qrUrl;
        img.alt = "QR Code";
        img.style.width = this.opt.width + "px";
        img.style.height = this.opt.height + "px";
        img.style.borderRadius = "8px";
        img.style.boxShadow = "0 4px 15px rgba(0,0,0,0.3)";
        this.el.appendChild(img);
    };

    window.QRCode = QRCode;
})();
